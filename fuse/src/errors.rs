use crate::prelude::*;
use std::ffi::OsString;

#[derive(Error, Debug)]
pub enum ObjectStoreError {
    #[error("Unsupported Object Type")]
    UnsupportedObjectType,

    #[error("{0:?} exists already, no --force given")]
    ObjectStoreExists(OsString),

    #[error("{0:?} exists and is not empty")]
    ObjectStoreForeignExists(OsString),

    #[error("{0:?} is not a directory")]
    ObjectStoreNoDir(OsString),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}
