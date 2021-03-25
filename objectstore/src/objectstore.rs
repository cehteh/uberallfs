use crate::prelude::*;

use libc;
use openat::Dir;
use rand::prelude::*;
use rand_core::OsRng;
use rand_hc::Hc128Rng;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::{fs::OpenOptions, path};

use lazy_static::lazy_static;
use regex::bytes::Regex;
use std::convert::TryInto;

use crate::identifier::{Flipbase64, Identifier, IdentifierBin};
use crate::identifier_kind::*;
use crate::object::Object;

pub struct Meta;

pub struct ObjectStore {
    version: u32,
    handle: Dir,
    objects: Dir,
    rng: Hc128Rng,
    //log: File, //TODO: logging 'dangerous' actions to be undone

    //PLANNED: pid: dir stack for all open dir handles (cwd/parents)

    //PLANNED: fd/object cache (drop handles when permissions get changed), MRU
    //PLANNED: identifier hash
}

impl ObjectStore {
    fn get_version(dir: &path::Path) -> Result<u32> {
        use std::io::{BufRead, BufReader};

        let mut version_name = path::PathBuf::from(dir);
        version_name.push("objectstore.version");

        let mut version_str: String = String::new();

        BufReader::new(OpenOptions::new().read(true).open(&version_name)?)
            .read_line(&mut version_str)?;

        version_str.pop();

        let version = version_str.parse::<u32>()?;
        trace!("version: {}", version);

        Ok(version)
    }

    pub(crate) fn open(dir: &path::Path) -> Result<ObjectStore> {
        let version = Self::get_version(dir)?;
        ensure!(
            version == crate::VERSION,
            ObjectStoreError::UnsupportedObjectStore(version)
        );

        let handle = Dir::open(dir)?;
        let objects = handle.sub_dir("objects")?;

        Ok(ObjectStore {
            version,
            handle,
            objects,
            rng: Hc128Rng::from_rng(OsRng)?,
        })
    }

    pub(crate) fn rng_identifier(&mut self) -> IdentifierBin {
        IdentifierBin(self.rng.gen())
    }

    pub(crate) fn import(&self, _archive: &OsStr) -> Result<Object> {
        unimplemented!()
    }

    // get the root identifier
    pub(crate) fn get_root_id(&self) -> Result<Identifier> {
        let root_link = self.objects.read_link("root")?;

        if let Some(root_name) = root_link.file_name() {
            trace!("root: {:?}", root_name);
            Identifier::from_flipbase64(Flipbase64(root_name.as_bytes().try_into()?))
        } else {
            bail!(ObjectStoreError::ObjectStoreFatal(String::from(
                "root directory not found"
            )))
        }
    }

    /// Takes a abbrevitated identifier as string and returns the full identifier object if exists
    pub(crate) fn identifier_lookup(&self, abbrev: &OsStr) -> Result<Identifier> {
        trace!("prefix: {:?}", abbrev);
        match abbrev.len() {
            len if len < 4 || len > 44 => {
                bail!(ObjectStoreError::InvalidIdentifier(String::from(
                    "abbrevitated identifiers must be between 4 to 44 characters in length",
                )))
            }
            len if len == 44 => {
                let mut path = path::PathBuf::from(OsStr::from_bytes(&abbrev.as_bytes()[..2]));
                path.push(abbrev);

                self.objects.metadata(path.as_path())?;
                //TODO: look into deleted objects / revive
                Identifier::from_flipbase64(Flipbase64(abbrev.as_bytes().try_into()?))
            }
            _ => {
                let path = path::PathBuf::from(OsStr::from_bytes(&abbrev.as_bytes()[..2]));

                let mut found: Option<OsString> = None;
                for entry in self.objects.list_dir(path.as_path())? {
                    let entry = entry?;
                    if entry.file_name().len() == 44 {
                        if entry.file_name().as_bytes()[..abbrev.len()] == *abbrev.as_bytes() {
                            if found == None {
                                found = Some(OsString::from(entry.file_name()));
                            } else {
                                bail!(ObjectStoreError::IdentifierAmbiguous(abbrev.into()));
                            }
                        }
                    }
                }
                //TODO: look into deleted objects / revive

                Identifier::from_flipbase64(Flipbase64(
                    found
                        .ok_or(ObjectStoreError::ObjectNotFound(abbrev.into()))?
                        .as_bytes()
                        .try_into()?,
                ))
            }
        }
    }

    /// Do full path lookups
    /// Paths can start with:
    ///  - a single slash, then path traversal starts at the root
    ///  - double slashes, followed by an abbrevitated Identifier, then the traversal starts there
    /// The path is traversed as much as possible, optionally storing the identifiers (parents) leading to that.
    /// Returns the finally found identifiers and the rest of the path thats is not existant.
    pub(crate) fn path_lookup(
        &self,
        path: Option<&OsStr>,
        parents: Option<&mut Vec<Identifier>>,
    ) -> Result<(Identifier, Option<path::PathBuf>)> {
        let (root, path): (Result<Identifier>, Option<&OsStr>) = match path {
            None => {
                // no path at all means root
                //if let Some(vec) = parents {
                //} TODO: push
                (self.get_root_id(), None)
            }
            Some(path) => {
                lazy_static! {
                    static ref PATH_RE: Regex =
                        Regex::new("^(?-u)(/{0,2})(([^/]*)/?(.*))$").unwrap();
                }

                match PATH_RE.captures(path.as_bytes()) {
                    Some(captures) => match captures.get(1) {
                        Some(slashes) => match slashes.as_bytes() {
                            b"//" => (
                                self.identifier_lookup(
                                    captures
                                        .get(3)
                                        .and_then(|c| Some(OsStr::from_bytes(&c.as_bytes())))
                                        .unwrap(),
                                ),
                                captures
                                    .get(4)
                                    .and_then(|c| Some(OsStr::from_bytes(&c.as_bytes()))),
                            ),
                            b"/" => (
                                self.get_root_id(),
                                captures
                                    .get(2)
                                    .and_then(|c| Some(OsStr::from_bytes(&c.as_bytes()))),
                            ),
                            _ => {
                                bail!(ObjectStoreError::ObjectStoreFatal(String::from(
                                    // reserved for future use
                                    "Paths w/o leading slash are not supported"
                                )))
                            }
                        },
                        None => unreachable!(), //TODO: can this happen?
                    },
                    None => bail!(ObjectStoreError::ObjectStoreFatal(String::from(
                        "Invalid PATH"
                    ))),
                }
            }
        };

        trace!("root: {:?}", root);

        let path = path.map(Self::normalize_path).transpose()?;

        trace!("path: {:?}", path);

        Ok((root?, None))
    }

    pub(crate) fn normalize_path(path: &OsStr) -> Result<OsString> {
        let mut new_path = path::PathBuf::new();
        for p in path::PathBuf::from(path).iter() {
            if p != "." {
                if p == ".." {
                    if !new_path.pop() {
                        bail!(ObjectStoreError::NoParent)
                    }
                } else {
                    new_path.push(p);
                }
            }
        };

        Ok(new_path.into_os_string())
    }

    pub(crate) fn open_metadata(
        &self,
        identifier: &Identifier,
        metadata: Meta,
        access: FileAccess,
    ) -> Result<Handle> {
        unimplemented!()
    }

    pub(crate) fn create_metadata(
        &self,
        identifier: &Identifier,
        metadata: Meta,
        perm: FilePermissions, // readwrite or readonly for immutable metadata
    ) -> Result<Handle> {
        //access: FileAccess, -> always readwrite
        unimplemented!()
    }

    pub(crate) fn open_link(
        &self,
        object: ParentLink,
        access: FileAccess,
        perm: FilePermissions,
        attr: FileAttributes,
    ) -> Result<Handle> {
        //Self::ensure_dir(object.0)?;
        unimplemented!()
    }

    pub(crate) fn open_file(&self, identifier: &Identifier, access: FileAccess) -> Result<Handle> {
        unimplemented!()
    }

    pub(crate) fn create_file(
        &self,
        identifier: &Identifier,
        parent: Option<ParentLink>,
        access: FileAccess,
        perm: FilePermissions,
        attr: FileAttributes,
    ) -> Result<Handle> {
        identifier.ensure_file()?;
        //Self::ensure_dir(object.0);
        unimplemented!()
    }

    pub(crate) fn create_link(&self, identifier: &Identifier, parent: ParentLink) -> Result<()> {
        parent.0.ensure_dir()?;
        unimplemented!()
    }

    // open dir is only for read, no access type needed
    pub(crate) fn open_directory(&self, identifier: &Identifier) -> Result<Handle> {
        identifier.ensure_dir()?;
        unimplemented!()
    }

    pub(crate) fn create_directory(
        &self,
        identifier: &Identifier,
        parent: Option<ParentLink>,
        perm: DirectoryPermissions,
    ) -> Result<()> {
        identifier.ensure_dir()?;
        let path = Path::new().push_identifier(identifier);
        info!("mkdir: {:?}", path.as_os_str());

        self.objects.create_dir(path.as_os_str(), perm.get())?;

        if let Some(parent) = parent {
            parent.0.ensure_dir()?;

            let path = Path::prefix(OsStr::new(".."))
                .push_identifier(parent.0)
                .push(parent.1);
            info!(
                "link: {:?} -> {:?}",
                path.as_os_str(),
                identifier.id_base64().0
            );
        }

        Ok(())
    }

    pub(crate) fn change_access(&self, identifier: &Identifier, access: FileAccess) -> Result<()> {
        unimplemented!()
    }

    pub(crate) fn change_attributes(
        &self,
        identifier: &Identifier,
        attr: FileAttributes,
    ) -> Result<()> {
        unimplemented!()
    }

    //pub fn remove_object // move to deleted (w/ link)

    //pub fn revive_object // move from deleted

    //pub fn cleanup_deleted // delete expired objects
    pub(crate) fn set_root(&self, identifier: &Identifier) -> Result<()> {
        identifier.ensure_dir()?;
        let path = Path::new().push_identifier(identifier);
        info!("set_root: {:?}", path.as_os_str());
        self.objects.remove_file("root");
        self.objects
            .symlink("root", path.as_os_str())
            .with_context(|| "failed to symlink root object")
    }
}

impl Drop for ObjectStore {
    fn drop(&mut self) {}
}

pub enum Handle {
    Dir(openat::Dir),
    File(std::fs::File),
}

// impl Handle
// change_access() etc

pub struct ParentLink<'a>(&'a Identifier, &'a OsStr);

pub struct Path(path::PathBuf);

impl Path {
    pub fn new() -> Self {
        Path(path::PathBuf::new())
    }

    pub fn prefix(prefix: &OsStr) -> Self {
        Path(path::PathBuf::from(prefix))
    }

    pub fn push(mut self, name: &OsStr) -> Self {
        self.0.push(name);
        self
    }

    pub fn push_identifier(mut self, identifier: &Identifier) -> Self {
        let bytes = identifier.id_base64().0;
        self.0.push(OsStr::from_bytes(&bytes[..2]));
        self.0.push(OsStr::from_bytes(&bytes));
        self
    }

    pub fn set_file_name(mut self, name: &OsStr) -> Self {
        self.0.set_file_name(name);
        self
    }

    pub fn set_extension(mut self, ext: &OsStr) -> Self {
        self.0.set_extension(ext);
        self
    }

    pub fn as_os_str(&self) -> &OsStr {
        self.0.as_os_str()
    }
}

/*
These are permissions/access flages of the objects in the objectstore, abstrated from host
filesystem implementation.

For the first implementation uberallfs is single user/group on the local objectstore and fuse
frontend only (This does not affect permissions and security globally).
 */

#[cfg(unix)]
pub struct FileAccess(libc::c_int);

#[cfg(unix)]
impl FileAccess {
    pub fn new() -> Self {
        FileAccess(0)
    }

    pub fn readonly(mut self) -> Self {
        assert_eq!(self.0, 0);
        self.0 = libc::O_RDONLY;
        self
    }

    pub fn writeonly(mut self) -> Self {
        assert_eq!(self.0, 0);
        self.0 = libc::O_WRONLY;
        self
    }

    pub fn readwrite(mut self) -> Self {
        assert_eq!(self.0, 0);
        self.0 = libc::O_RDWR;
        self
    }

    pub fn append(mut self) -> Self {
        assert_eq!(self.0, 0);
        self.0 = self.0 | libc::O_APPEND;
        self
    }

    pub fn extra_flags(mut self, flags: libc::c_int) -> Self {
        assert_eq!(self.0, 0);
        self.0 = self.0 | flags;
        self
    }

    fn get(self) -> libc::c_int {
        self.0 | libc::O_CLOEXEC
    }

    fn get_no_cloexec(self) -> libc::c_int {
        self.0
    }
}

#[cfg(unix)]
pub struct FilePermissions(libc::mode_t);

#[cfg(unix)]
impl FilePermissions {
    pub fn new() -> Self {
        FilePermissions(0)
    }

    pub fn read(mut self) -> Self {
        self.0 = self.0 | libc::S_IRUSR | libc::S_IRGRP;
        self
    }

    pub fn write(mut self) -> Self {
        self.0 = self.0 | libc::S_IWUSR | libc::S_IWGRP;
        self
    }

    pub fn full(mut self) -> Self {
        self.0 = self.0 | libc::S_IRUSR | libc::S_IRGRP | libc::S_IWUSR | libc::S_IWGRP;
        self
    }

    fn get(self) -> libc::mode_t {
        self.0
    }
}

#[cfg(unix)]
pub struct FileAttributes(libc::mode_t);

#[cfg(unix)]
impl FileAttributes {
    pub fn new() -> Self {
        FileAttributes(0)
    }

    pub fn execute(mut self) -> Self {
        self.0 = self.0 | libc::S_IXUSR | libc::S_IXGRP;
        self
    }

    fn get(self) -> libc::mode_t {
        self.0
    }
}

#[cfg(unix)]
pub struct DirectoryPermissions(libc::mode_t);

#[cfg(unix)]
impl DirectoryPermissions {
    pub fn new() -> Self {
        DirectoryPermissions(0)
    }

    pub fn list(mut self) -> Self {
        self.0 = self.0 | libc::S_IRUSR | libc::S_IRGRP;
        self
    }

    pub fn read(mut self) -> Self {
        self.0 = self.0 | libc::S_IXUSR | libc::S_IXGRP;
        self
    }

    pub fn change(mut self) -> Self {
        self.0 = self.0 | libc::S_IWUSR | libc::S_IWGRP;
        self
    }

    pub fn full(mut self) -> Self {
        self.0 = self.0
            | libc::S_IRUSR
            | libc::S_IRGRP
            | libc::S_IXUSR
            | libc::S_IXGRP
            | libc::S_IWUSR
            | libc::S_IWGRP;
        self
    }

    fn get(self) -> libc::mode_t {
        self.0
    }
}
