use crate::prelude::*;

use std::convert::TryInto;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::prelude::RawFd;
use std::{fs::OpenOptions, path::Path, path::PathBuf};

use lazy_static::lazy_static;
use openat::{Dir, Metadata};
use regex::bytes::Regex;

use uberall::UberAll;

use crate::{objectpath, Flipbase64, Handle, Identifier, IdentifierBin, Object, ObjectPath};

pub struct Meta;

pub struct ObjectStore {
    #[allow(dead_code)]
    version: u32,
    #[allow(dead_code)]
    handle: Dir,
    objects: Dir,

    uberall: UberAll,
    //log: File, //TODO: logging 'dangerous' actions to be undone

    //PLANNED: pid: dir stack for all open dir handles (cwd/parents)

    //PLANNED: fd/object cache (drop handles when permissions get changed), MRU
    //PLANNED: identifier hash
}

/// The ObjectStore
impl ObjectStore {
    /// reads the objectstore version on disk. This defines the layout of the files on disk.
    /// Version '0' is a everlasting development and incompatible with anything else version.
    fn get_version(dir: &Path) -> Result<u32> {
        use std::io::{BufRead, BufReader};

        let mut version_name = PathBuf::from(dir);
        version_name.push("objectstore.version");

        let mut version_str: String = String::new();

        BufReader::new(OpenOptions::new().read(true).open(&version_name)?).read_line(&mut version_str)?;

        version_str.pop();

        let version = version_str.parse::<u32>()?;
        trace!("version: {}", version);

        Ok(version)
    }

    /// Opens an ObjectStore at the given path.
    pub fn open(dir: &Path) -> Result<ObjectStore> {
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
            uberall: UberAll::new()?,
        })
    }

    /// Returns an all-random binary representaton of an Object Identifier.
    pub(crate) fn rng_identifier(&mut self) -> IdentifierBin {
        IdentifierBin(self.uberall.rng_gen())
    }

    pub(crate) fn import(&self, _archive: &OsStr) -> Result<Object> {
        unimplemented!()
    }

    /// Return raw file descriptor of the objects dir
    pub fn get_objects_fd(&self) -> RawFd {
        self.objects.as_raw_fd()
    }

    /// Return the Identifier of the ObjectStores root Object
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
            len if !(4..=44).contains(&len) => {
                bail!(ObjectStoreError::InvalidIdentifier(String::from(
                    "abbrevitated identifiers must be between 4 to 44 characters in length",
                )))
            }
            len if len == 44 => {
                let path = objectpath::from_bytes(&abbrev.as_bytes()[..2]).join(abbrev);
                self.objects.metadata(path.as_os_str())?;
                //TODO: look into deleted objects / revive
                Identifier::from_flipbase64(Flipbase64(abbrev.as_bytes().try_into()?))
            }
            _ => {
                let path = objectpath::from_bytes(&abbrev.as_bytes()[..2]);

                let mut found: Option<OsString> = None;
                for entry in self.objects.list_dir(path.as_os_str())? {
                    let entry = entry?;
                    if entry.file_name().len() == 44
                        && entry.file_name().as_bytes()[..abbrev.len()] == *abbrev.as_bytes()
                    {
                        if found == None {
                            found = Some(OsString::from(entry.file_name()));
                        } else {
                            bail!(ObjectStoreError::IdentifierAmbiguous(abbrev.into()));
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
    ///  - an abbrevitated Identifier followed by a double slash '//', then the traversal starts there
    /// The path is traversed as much as possible, optionally storing the identifiers (parents) leading to that.
    /// Returns the finally found identifiers and the rest of the path thats is not existant.
    pub fn path_lookup(
        &self,
        path: &Path,
        parents: Option<&mut Vec<Identifier>>,
    ) -> Result<(Identifier, PathBuf)> {
        if path.as_os_str() == "" {
            Ok((self.get_root_id()?, PathBuf::new()))
        } else {
            lazy_static! {
                static ref PATH_RE: Regex = Regex::new(r"^(?:([^/]{4,44})/|)/(.*)").unwrap();
            }

            let (root, mut path) =
                if let Some(captures) = PATH_RE.captures(path.as_os_str().as_bytes()) {
                    let root;
                    let id: &OsStr = OsStrExt::from_bytes(
                        if let Some(capture) = captures.get(1) {
                            capture.as_bytes()
                        } else {
                            root = self.get_root_id()?;
                            &root.id_base64().0
                        }
                    );
                    (
                        self.identifier_lookup(id)?,
                        objectpath::from_bytes(captures.get(2).unwrap().as_bytes())
                    )
                } else {
                    bail!(ObjectStoreError::ObjectStoreFatal(String::from(
                        "Invalid Path"
                    )))
                };

            path.normalize()?;

            self.traverse_path(root, path, parents)
        }
    }

    /// Walk the path starting at root following existing elements
    pub(crate) fn traverse_path(
        &self,
        mut root: Identifier,
        path: PathBuf,
        _parents: Option<&mut Vec<Identifier>>,
    ) -> Result<(Identifier, PathBuf)> {
        trace!("traverse: {:?}", &path);
        //TODO: track parents
        let mut out = PathBuf::new();

        let mut still_going = true;
        for p in path.iter() {
            trace!("traverse element: {:?}", &p);
            let subobject = SubObject(&root, p);
            if still_going {
                match self.sub_object_id(&subobject) {
                    Ok(r) => {
                        trace!("subobject: ok {:?}", &r);
                        root = r;
                    }
                    x => {
                        trace!("subobject: fail {:?}", &x);

                        still_going = false;
                        out.push(p);
                    }
                }
            } else {
                out.push(p);
            }
        }

        Ok((root, out))
    }

    /// get the identifier of a sub-object
    pub fn sub_object_id(&self, sub_object: &SubObject) -> Result<Identifier> {
        sub_object.0.ensure_dir()?;

        let r = self.objects.read_link(&sub_object.to_pathbuf())?;

        Identifier::from_flipbase64(Flipbase64(
            r.as_os_str().as_bytes()[crate::RESERVED_PREFIX.len()..].try_into()?,
        ))
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
        object: SubObject,
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
        parent: Option<SubObject>,
        access: FileAccess,
        perm: FilePermissions,
        attr: FileAttributes,
    ) -> Result<Handle> {
        identifier.ensure_file()?;
        //Self::ensure_dir(object.0);
        unimplemented!()
    }

    pub(crate) fn create_link(&self, identifier: &Identifier, parent: SubObject) -> Result<()> {
        parent.0.ensure_dir()?;

        let source = parent.to_pathbuf();
        let mut dest = PathBuf::new();
        dest.push_link(identifier);

        trace!(
            "mkdir link: {:?} -> {:?}",
            source.as_os_str(),
            dest.as_os_str()
        );

        self.objects
            .symlink(source.as_os_str(), dest.as_os_str())
            .with_context(|| "failed to symlink new dir")
    }

    // open dir is only for read, no access type needed
    pub(crate) fn open_directory(&self, identifier: &Identifier) -> Result<Handle> {
        identifier.ensure_dir()?;
        unimplemented!()
    }

    pub(crate) fn create_directory(
        &self,
        identifier: &Identifier,
        parent: Option<SubObject>,
        perm: DirectoryPermissions,
    ) -> Result<()> {
        identifier.ensure_dir()?;
        let mut path = PathBuf::new();
        path.push_identifier(identifier);
        info!("create_directory: {:?}", path.as_os_str());

        self.objects.create_dir(path.as_os_str(), perm.get())?;

        if let Some(parent) = parent {
            parent.0.ensure_dir()?;

            let path = PathBuf::from("..").push_identifier(parent.0).push(parent.1);
            info!("link: {:?} -> {:?}", path, identifier.id_base64().0);
        }

        Ok(())
    }

    pub(crate) fn change_access(&self, identifier: &Identifier, access: FileAccess) -> Result<()> {
        unimplemented!()
    }

    pub(crate) fn change_attributes(&self, identifier: &Identifier, attr: FileAttributes) -> Result<()> {
        unimplemented!()
    }

    pub fn object_metadata(&self, identifier: &Identifier) -> io::Result<Metadata> {
        self.objects.metadata(identifier.to_pathbuf().as_path())
    }

    //pub fn remove_object // move to deleted (w/ link)

    //pub fn revive_object // move from deleted

    //pub fn cleanup_deleted // delete expired objects
    pub(crate) fn set_root(&self, identifier: &Identifier) -> Result<()> {
        identifier.ensure_dir()?;
        let mut path = PathBuf::new();
        path.push_identifier(identifier);
        info!("set_root: {:?}", path.as_os_str());
        self.objects.remove_file("root").ok();
        self.objects
            .symlink("root", path.as_os_str())
            .with_context(|| "failed to symlink root object")
    }
}

impl Drop for ObjectStore {
    fn drop(&mut self) {}
}

/// identifier/name pair for a subobject in a directory
pub struct SubObject<'a>(pub &'a Identifier, pub &'a OsStr);

impl SubObject<'_> {
    #[inline]
    pub fn to_pathbuf(&self) -> PathBuf {
        PathBuf::new().push_identifier(self.0).join(self.1)
    }
}

/*
These are permissions/access flages of the objects in the objectstore, abstrated from host
filesystem implementation.

For the first implementation uberallfs is single user/group on the local objectstore and fuse
frontend only (This does not affect permissions and security globally).
 */

#[cfg(unix)]
#[derive(Default)]
pub struct FileAccess(libc::c_int);

#[cfg(unix)]
impl FileAccess {
    pub fn new() -> Self {
        FileAccess::default()
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
        self.0 |= libc::O_APPEND;
        self
    }

    pub fn extra_flags(mut self, flags: libc::c_int) -> Self {
        assert_eq!(self.0, 0);
        self.0 |= flags;
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
#[derive(Default)]
pub struct FilePermissions(libc::mode_t);

#[cfg(unix)]
impl FilePermissions {
    pub fn new() -> Self {
        FilePermissions::default()
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
#[derive(Default)]
pub struct FileAttributes(libc::mode_t);

#[cfg(unix)]
impl FileAttributes {
    pub fn new() -> Self {
        FileAttributes::default()
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
#[derive(Default)]
pub struct DirectoryPermissions(libc::mode_t);

#[cfg(unix)]
impl DirectoryPermissions {
    pub fn new() -> Self {
        DirectoryPermissions::default()
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
        self.0 = self.0 | libc::S_IRUSR | libc::S_IRGRP | libc::S_IXUSR | libc::S_IXGRP | libc::S_IWUSR | libc::S_IWGRP;
        self
    }

    fn get(self) -> libc::mode_t {
        self.0
    }
}
