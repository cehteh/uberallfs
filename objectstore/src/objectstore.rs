#[allow(unused_imports)]
use log::{debug, error, info, trace};

use libc;
use openat::Dir;
use rand::prelude::*;
use rand_core::OsRng;
use rand_hc::Hc128Rng;
use std::ffi::OsStr;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use crate::identifier::{Flipbase64, Identifier, IdentifierBin};
use crate::identifier_kind::*;
use crate::object::Object;

pub struct ObjectStore {
    handle: Dir,
    objects: Dir, //Weakref to cache
    rng: Hc128Rng,
    //PLANNED: fd cache, MRU
}

fn identifier_to_path(identifier: &Identifier) -> Box<Path> {
    let bytes = identifier.id_base64().0;
    Path::new(OsStr::from_bytes(&bytes[..2]))
        .join(OsStr::from_bytes(&bytes))
        .into_boxed_path()
}

impl ObjectStore {
    pub(crate) fn open(dir: &Path) -> io::Result<ObjectStore> {
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

    pub(crate) fn make_directory(
        &self,
        identifier: &Identifier,
        perm: DirectoryPermissions,
    ) -> io::Result<()> {
        assert_eq!(identifier.object_type(), ObjectType::Directory);
        let path = identifier_to_path(identifier);
        info!("mkdir: {:?}", &*path);
        self.objects.create_dir(&*path, perm.get())
    }

    pub(crate) fn set_root(&self, identifier: &Identifier) -> io::Result<()> {
        assert_eq!(identifier.object_type(), ObjectType::Directory);
        let path = identifier_to_path(&identifier);
        info!("setroot: {:?}", &*path);
        self.objects.remove_file("root").ok();
        self.objects.symlink("root", &*path)
    }
}

impl Drop for ObjectStore {
    fn drop(&mut self) {}
}

/*
These are permissions of the objects in the objectstore, abstrated from host filesystem
permissions, not uberallfs permissions.

For the first implementation uberallfs is single user/group on the local objectstore and fuse
frontend only (This does not affect permissions and security globally).
*/
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
