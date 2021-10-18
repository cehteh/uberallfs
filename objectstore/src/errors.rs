use std::ffi::OsString;

use uberall::thiserror::{self, Error};

use crate::prelude::*;
use crate::identifier_kind::*;

#[derive(Error, Debug)]
pub enum ObjectStoreError {
    #[error("Unsupported ObjectStore version {0}")]
    UnsupportedObjectStore(u32),

    #[error("fatal objectstore error: {0}")]
    ObjectStoreFatal(String),

    #[error("Argument error: {0}")]
    OptArgError(String),

    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),

    #[error("Ambigous identifier: {0:?}")]
    IdentifierAmbiguous(OsString),

    #[error("Unsupported Object Type: {0:?}")]
    UnsupportedObjectType((ObjectType, SharingPolicy, Mutability)),

    #[error("{0:?} exists already, no --force given")]
    ObjectStoreExists(OsString),

    #[error("{0:?} exists and is not empty")]
    ObjectStoreForeignExists(OsString),

    #[error("{0:?} is not a directory")]
    ObjectStoreNoDir(OsString),

    #[error("Wrong object type: got '{have:?}' expected '{want:?}'")]
    ObjectType { have: ObjectType, want: ObjectType },

    #[error("Can not traverse into a parent object")]
    NoParent,

    #[error("Could not acquire lock on the objectstore")]
    NoLock,

    #[error("Object {0:?} exists already")]
    ObjectExists(OsString),

    #[error("Object {0:?} not found")]
    ObjectNotFound(OsString),

    #[error("Illegal file name: {0:?}")]
    IllegalFileName(OsString),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}
