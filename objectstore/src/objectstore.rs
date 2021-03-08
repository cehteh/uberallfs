use openat::Dir;
use std::path::{Path, PathBuf};
use std::io::{self, Error, ErrorKind};
use std::ffi::OsStr;

use crate::object::{Object, IdentifierType, ObjectType};

pub struct ObjectStore {
    handle: Dir,
}

impl ObjectStore {
    pub fn open(dir: &Path) -> io::Result<ObjectStore> {
        Ok(ObjectStore {
            handle: Dir::open(dir)?,
        })
    }

    pub fn import(&self, archive: &OsStr) -> io::Result<Object> {
        Err(Error::new(ErrorKind::Other, "unimplemented"))
    }

    pub fn create_object(&self, id_type: IdentifierType, file_type: ObjectType) -> io::Result<Object> {
        Err(Error::new(ErrorKind::Other, "unimplemented"))
    }

    pub fn set_root(&self, root: &Object) -> io::Result<()> {
        Err(Error::new(ErrorKind::Other, "unimplemented"))
    }
}

impl Drop for ObjectStore {
    fn drop(&mut self) {}
}
