use openat::Dir;
use rand::prelude::*;
use rand_core::OsRng;
use rand_hc::Hc128Rng;
use std::ffi::OsStr;
use std::io;
use std::path::Path;

use crate::object::{IdType, Object};

pub struct ObjectStore {
    handle: Dir,
    rng: Hc128Rng,
}

impl ObjectStore {
    pub(crate) fn open(dir: &Path) -> io::Result<ObjectStore> {
        Ok(ObjectStore {
            handle: Dir::open(dir)?,
            rng: Hc128Rng::from_rng(OsRng)?,
        })
    }

    pub fn rng_gen(&mut self) -> IdType {
        self.rng.gen()
    }

    pub fn import(&self, _archive: &OsStr) -> io::Result<Object> {
        unimplemented!()
    }

    pub fn set_root(&self, _root: &Object) -> io::Result<()> {
        unimplemented!()
    }
}

impl Drop for ObjectStore {
    fn drop(&mut self) {}
}
