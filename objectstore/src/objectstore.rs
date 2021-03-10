use openat::Dir;
use rand::prelude::*;
use rand_core::OsRng;
use rand_hc::Hc128Rng;
use std::ffi::OsStr;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use crate::object::{
    IdentifierType::{self, *},
    Object, ObjectType,
};

pub struct ObjectStore {
    handle: Dir,
    rng: Hc128Rng,
}

impl ObjectStore {
    pub fn open(dir: &Path) -> io::Result<ObjectStore> {
        Ok(ObjectStore {
            handle: Dir::open(dir)?,
            rng: Hc128Rng::from_rng(OsRng)?,
        })
    }

    pub fn create_object(
        &mut self,
        id_type: IdentifierType,
        object_type: ObjectType,
    ) -> io::Result<Object> {
        match id_type {
            PrivateMutable => Object::create_private_mutable(object_type, self.rng.gen()),

            _ => unimplemented!(),
        }
    }

    pub fn import(&self, archive: &OsStr) -> io::Result<Object> {
        unimplemented!()
    }

    pub fn set_root(&self, root: &Object) -> io::Result<()> {
        unimplemented!()
    }
}

impl Drop for ObjectStore {
    fn drop(&mut self) {}
}
