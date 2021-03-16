#[allow(unused_imports)]
use log::{debug, error, info, trace};

use libc;
use rand::prelude::*;
use rand_core::OsRng;
use rand_hc::Hc128Rng;
use std::ffi::OsStr;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path;
use openat::Dir;

use crate::identifier::{Flipbase64, Identifier, IdentifierBin};
use crate::identifier_kind::*;
use crate::object::Object;

pub struct ObjectStore {
    handle: Dir,
    objects: Dir, //Weakref to cache
    rng: Hc128Rng,
    //PLANNED: fd cache (drop handles when permissions get changed), MRU
}

impl ObjectStore {
    pub(crate) fn open(dir: &path::Path) -> io::Result<ObjectStore> {
        let handle = Dir::open(dir)?;
        let objects = handle.sub_dir("objects")?;
        Ok(ObjectStore {
            handle,
            objects,
            rng: Hc128Rng::from_rng(OsRng)?,
        })
    }

    pub(crate) fn rng_gen(&mut self) -> IdentifierBin {
        IdentifierBin(self.rng.gen())
    }

    pub(crate) fn import(&self, _archive: &OsStr) -> io::Result<Object> {
        unimplemented!()
    }

/*
    pub(crate) fn open_metadata(
        &self,
        identifier: &Identifier,
        metadata: Meta,
        access: FileAccess,
    ) -> io::Result<Handle> {
        unimplemented!()
    }

    pub(crate) fn create_metadata(
        &self,
        identifier: &Identifier,
        metadata: Meta,
        perm: FilePermissions, // readwrite or readonly for immutable metadata
    ) -> io::Result<Handle> {
        access: FileAccess, -> always readwrite
        unimplemented!()
    }
     */

    pub(crate) fn open_file(
        &self,
        identifier: &Identifier,
        access: FileAccess,
        perm: FilePermissions,
        attr: FileAttributes,
    ) -> io::Result<Handle> {
        unimplemented!()
    }

    pub(crate) fn create_file(
        &self,
        identifier: &Identifier,
        access: FileAccess,
        perm: FilePermissions,
        attr: FileAttributes,
    ) -> io::Result<Handle> {
        unimplemented!()
    }

    /*
    pub(crate) fn open_directory(
        &self,
        identifier: &Identifier,
    ) -> io::Result<Handle::Dir> {
        unimplemented!()
    }
*/
    pub(crate) fn create_directory(
        &self,
        identifier: &Identifier,
        perm: DirectoryPermissions,
    ) -> io::Result<()> {
        assert_eq!(identifier.object_type(), ObjectType::Directory);

        let path = Path::new().push_identifier(identifier);
        info!("mkdir: {:?}", path.as_os_str());
        self.objects.create_dir(path.as_os_str(), perm.get())
    }

    pub(crate) fn change_access(
        &self,
        identifier: &Identifier,
        access: FileAccess,
    ) -> io::Result<()> {
        unimplemented!()
    }


    //pub fn remove_object // move to deleted

    //pub fn revive_object // move from deleted

    //pub fn cleanup_deleted // delete expired objects

    pub(crate) fn set_root(&self, identifier: &Identifier) -> io::Result<()> {
        assert_eq!(identifier.object_type(), ObjectType::Directory);
        let path = Path::new().push_identifier(identifier);
        info!("set_root: {:?}", path.as_os_str());
        self.objects.remove_file("root").ok();
        self.objects.symlink("root", path.as_os_str())
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
// chance_access() etc



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
