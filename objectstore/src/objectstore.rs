use openat::Dir;
use std::io;
use std::path::{Path, PathBuf};

pub struct ObjectStore {
    handle: Dir,
}

impl ObjectStore {
    pub fn open(dir: &Path) -> io::Result<ObjectStore> {
        Ok(ObjectStore {
            handle: Dir::open(dir)?,
        })
    }
}

impl Drop for ObjectStore {
    fn drop(&mut self) {}
}
