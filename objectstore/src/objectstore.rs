use std::convert::TryInto;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::prelude::RawFd;
use std::{fs::OpenOptions, path::Path, path::PathBuf};

use openat_ct as openat;
use openat::{Dir, DirIter, Entry, Metadata, SimpleType};
use regex::bytes::Regex;
use uberall::{lazy_static::lazy_static, libc, UberAll};
use itertools::repeat_n;

use crate::prelude::*;
use crate::{
    lock_fd, objectpath, Flipbase64, Handle, Identifier, IdentifierBin, LockingMethod, Object,
    ObjectPath,
};

pub struct Meta;

#[derive(Debug)]
pub struct ObjectStore {
    #[allow(dead_code)]
    pub(crate) version: u32,
    //#[allow(dead_code)]
    // handle:             Dir, // FIXME: remove handle, needs only objects and lock on that
    pub(crate) objects: Dir,

    pub(crate) uberall: UberAll,
    /* TODO: log: File, logging 'dangerous' actions to be undone
     * PLANNED: pid: dir stack for all open dir handles (cwd/parents)
     * PLANNED: fd/object cache (drop handles when permissions get changed), MRU
     * PLANNED: identifier hash */
}

/// The ObjectStore
impl ObjectStore {
    /// reads the objectstore version on disk. This defines the layout of the files on
    /// disk. Version '0' is a everlasting development and incompatible with any other
    /// version (including itself from former development cycles).
    fn get_version(dir: &Path) -> Result<u32> {
        use std::io::{BufRead, BufReader};

        let mut version_name = PathBuf::from(dir);
        version_name.push("objects/version");

        let mut version_str: String = String::new();

        BufReader::new(OpenOptions::new().read(true).open(&version_name)?)
            .read_line(&mut version_str)?;

        version_str.pop();

        let version = version_str.parse::<u32>()?;

        Ok(version)
    }

    /// Opens an ObjectStore at the given path.
    pub fn open(dir: &Path, locking_method: LockingMethod) -> Result<ObjectStore> {
        let mut objects_dir = PathBuf::from(dir);
        objects_dir.push("objects");
        let objects = Dir::flags().open(&objects_dir)?;
        lock_fd(&objects, locking_method)?;

        let version = Self::get_version(dir)?;
        debug!("open {:?}, version: {}", dir, version);
        if version != crate::VERSION {
            return Err(ObjectStoreError::UnsupportedObjectStore(version).into());
        }

        Ok(ObjectStore {
            version,
            objects,
            uberall: UberAll::new()?,
        })
    }

    /// Returns an all-random binary representaton of an Object Identifier.
    pub(crate) fn rng_identifier(&self) -> IdentifierBin {
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
            Err(ObjectStoreError::ObjectStoreFatal(String::from("root directory not found")).into())
        }
    }

    /// Takes a abbrevitated identifier as string and returns the full
    /// identifier object if exists
    pub(crate) fn identifier_lookup(&self, abbrev: &OsStr) -> Result<Identifier> {
        trace!("prefix: {:?}", abbrev);
        match abbrev.len() {
            len if !(4..=44).contains(&len) => {
                Err(ObjectStoreError::InvalidIdentifier(String::from(
                    "abbrevitated identifiers must be between 4 to 44 characters in length",
                ))
                .into())
            }
            len if len == 44 => {
                let path = objectpath::from_bytes(&abbrev.as_bytes()[..2]).join(abbrev);
                self.objects.metadata(path.as_os_str())?;
                // TODO: look into deleted objects / revive
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
                            return Err(ObjectStoreError::IdentifierAmbiguous(abbrev.into()).into());
                        }
                    }
                }
                // TODO: look into deleted objects / revive

                Identifier::from_flipbase64(Flipbase64(
                    found
                        .ok_or_else(|| ObjectStoreError::ObjectNotFound(abbrev.into()))?
                        .as_bytes()
                        .try_into()?,
                ))
            }
        }
    }

    /// Do full path lookups
    /// Paths can start with:
    ///  - a single slash, then path traversal starts at the root
    ///  - an abbrevitated Identifier followed by a double slash '//', then the
    ///    traversal starts there
    /// The path is traversed as much as possible, optionally storing the
    /// identifiers (parents) leading to that. Returns the finally found
    /// identifiers and the rest of the path thats is not existant.
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
                    let id: &OsStr = OsStrExt::from_bytes(if let Some(capture) = captures.get(1) {
                        capture.as_bytes()
                    } else {
                        root = self.get_root_id()?;
                        &root.id_base64().0
                    });
                    (
                        self.identifier_lookup(id)?,
                        objectpath::from_bytes(captures.get(2).unwrap().as_bytes()),
                    )
                } else {
                    return Err(
                        ObjectStoreError::ObjectStoreFatal(String::from("Invalid Path")).into(),
                    );
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
        // TODO: track parents
        let mut out = PathBuf::new();

        let mut still_going = true;
        for p in path.iter() {
            let subobject = SubObject(&root, p);
            if still_going {
                trace!("traverse element: {:?}", &p);
                match self.sub_object_id(&subobject) {
                    Ok(r) => {
                        trace!("subobject: ok {:?}", &r);
                        root = r;
                    }
                    Err(err) => match err.downcast_ref::<io::Error>().map(io::Error::kind) {
                        Some(io::ErrorKind::NotFound) => {
                            // execpted case, just push path together
                            trace!("subobject: {:?}", &err);
                            still_going = false;
                            out.push(p);
                        }
                        Some(_) | None => {
                            // unexpected error
                            error!("subobject: {:?}", &err);
                            return Err(err);
                        }
                    },
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
            r.as_os_str().as_bytes()[crate::RESERVED_PREFIX.len() + 1..].try_into()?,
        ))
    }

    pub(crate) fn open_metadata(
        &self,
        _identifier: &Identifier,
        _metadata: Meta,
        _access: FileAccess,
    ) -> Result<Handle> {
        unimplemented!()
    }

    pub(crate) fn create_metadata(
        &self,
        _identifier: &Identifier,
        _metadata: Meta,
        _perm: FilePermissions, // readwrite or readonly for immutable metadata
    ) -> Result<Handle> {
        // access: FileAccess, -> always readwrite
        unimplemented!()
    }

    pub(crate) fn open_link(
        &self,
        _object: SubObject,
        _access: FileAccess,
        _perm: FilePermissions,
        _attr: FileAttributes,
    ) -> Result<Handle> {
        // Self::ensure_dir(object.0)?;
        unimplemented!()
    }

    pub(crate) fn open_file(
        &self,
        _identifier: &Identifier,
        _access: FileAccess,
    ) -> Result<Handle> {
        unimplemented!()
    }

    pub(crate) fn create_file(
        &self,
        identifier: &Identifier,
        _parent: Option<SubObject>,
        _access: FileAccess,
        _perm: FilePermissions,
        _attr: FileAttributes,
    ) -> Result<Handle> {
        identifier.ensure_file()?;
        // Self::ensure_dir(object.0);
        unimplemented!()
    }

    /// Create a link from 'parent' directory (identifier/name pair) to the
    /// given identifier
    pub(crate) fn create_link(&self, identifier: &Identifier, parent: SubObject) -> Result<()> {
        parent.0.ensure_dir()?;

        let source = parent.to_pathbuf();
        let mut dest = PathBuf::new();
        dest.push_link(identifier);

        let file_name = source.file_name().unwrap();
        if file_name.as_bytes().len() >= crate::RESERVED_PREFIX.len()
            && file_name.as_bytes()[..crate::RESERVED_PREFIX.len()] == crate::RESERVED_PREFIX
        {
            warn!("link: illegal file name: {:?}", &file_name);
            Err(ObjectStoreError::IllegalFileName(file_name.into()).into())
        } else {
            trace!("link: {:?} -> {:?}", source.as_os_str(), dest.as_os_str());

            self.objects
                .symlink(source.as_os_str(), dest.as_os_str())
                .map_err(|e| e.into())
        }
    }

    /// Opens a Dir handle to an Directory, identified by 'identifier'
    pub(crate) fn open_directory(&self, identifier: &Identifier) -> io::Result<Dir> {
        self.objects.sub_dir(identifier.to_pathbuf().as_path())
    }

    /// Opens a DirIter handle to an Directory, identified by 'identifier'.
    pub(crate) fn directory_iter(&self, identifier: &Identifier) -> io::Result<DirIter> {
        self.objects.list_dir(identifier.to_pathbuf().as_path())
    }

    /// Returns an iterator listing name/identifier pairs for a directory
    // TODO: make return type a struct DirEntry(CString, Identifier)
    pub fn list_directory(
        &self,
        identifier: &Identifier,
    ) -> io::Result<impl Iterator<Item = (PathBuf, Identifier)>> {
        let dir = self.open_directory(identifier)?;

        Ok(dir.list_self()?.filter_map(move |item| match item {
            Ok(Entry {
                name,
                file_type: Some(SimpleType::Symlink),
                ..
            }) => {
                let name = PathBuf::from(OsStr::from_bytes(name.to_bytes()));
                let identifier = Identifier::from_filename(&dir.read_link(&name).ok()?).ok()?;
                Some((name, identifier))
            }
            _ => None,
        }))
    }

    /// Creates a directory for an 'identifier'.
    pub(crate) fn create_directory(
        &self,
        identifier: &Identifier,
        perm: DirectoryPermissions,
    ) -> Result<()> {
        identifier.ensure_dir()?;
        let mut path = PathBuf::new();
        path.push_identifier(identifier);
        info!("create_directory: {:?}", path.as_os_str());

        self.objects.create_dir(path.as_os_str(), perm.get())?;
        Ok(())
    }

    pub(crate) fn change_access(
        &self,
        _identifier: &Identifier,
        _access: FileAccess,
    ) -> Result<()> {
        unimplemented!()
    }

    pub(crate) fn change_attributes(
        &self,
        _identifier: &Identifier,
        _attr: FileAttributes,
    ) -> Result<()> {
        unimplemented!()
    }

    /// Returns the underlying metadata for the identifier itself
    pub fn object_metadata(&self, identifier: &Identifier) -> io::Result<Metadata> {
        self.objects.metadata(identifier.to_pathbuf().as_path())
    }

    // pub fn remove_object // move to deleted (w/ link)

    // pub fn revive_object // move from deleted

    // pub fn cleanup_deleted // delete expired objects

    /// Registers the objectstores root directory to 'identifier'.
    pub(crate) fn set_root(&self, identifier: &Identifier) -> Result<()> {
        identifier.ensure_dir()?;
        let mut path = PathBuf::new();
        path.push_identifier(identifier);
        info!("set_root: {:?}", path.as_os_str());
        self.objects.remove_file("root").ok();
        self.objects
            .symlink("root", path.as_os_str())
            .map_err(|e| e.into())
    }

    /// Returns an iterator over all objects in the store
    // PLANNED: return -> impl Iterator<Item = Result<Identifier, ErrorWithContext>> later, fsck may need this
    pub fn all_objects(&self) -> impl Iterator<Item = Identifier> + '_ {
        const URL_SAFE_ENCODE: &[u8; 64] =
            &*b"ABCDEFGHIJLKMNOPQRSTUVWXYZabcdefghijlkmnopqrstuvwxyz0123456789-_";

        URL_SAFE_ENCODE
            .iter()
            .flat_map(|c| repeat_n(c, URL_SAFE_ENCODE.len()))
            .zip(URL_SAFE_ENCODE.iter().cycle())
            .flat_map(move |(a, b)| {
                let dirname = PathBuf::from(OsStr::from_bytes(&[*a, *b]));
                self.objects
                    .list_dir(&dirname)
                    .into_iter()
                    .flatten()
                    .filter_map(|r| {
                        if let Ok(entry) = r {
                            Some(Identifier::from_filename(Path::new(entry.file_name())).ok()?)
                        } else {
                            None
                        }
                    })
            })
    }
}

/// identifier/name pair for a subobject in a directory
#[derive(Debug)]
pub struct SubObject<'a>(pub &'a Identifier, pub &'a OsStr);

impl SubObject<'_> {
    #[inline]
    pub fn to_pathbuf(&self) -> PathBuf {
        PathBuf::new().push_identifier(self.0).join(self.1)
    }
}

// These are permissions/access flages of the objects in the objectstore,
// abstrated from host filesystem implementation.
//
// For the first implementation uberallfs is single user/group on the local
// objectstore and fuse frontend only (This does not affect permissions and
// security globally).

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
