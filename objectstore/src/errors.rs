use crate::prelude::*;
use std::ffi::OsString;

use crate::identifier_kind;

#[derive(Error, Debug)]
pub enum ObjectStoreError {
    #[error("Unsupported ObjectStore version {0}")]
    UnsupportedObjectStore(u32),

    #[error("fatal objectstore error {0}")]
    ObjectStoreFatal(String),

    #[error("conflicting arguments: {0}")]
    OptArgConflict(String), //TODO: OptArgError

    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),

    #[error("Ambigous identifier: {0:?}")]
    IdentifierAmbiguous(OsString),

    #[error("Unsupported Object Type")]
    UnsupportedObjectType,

    #[error("{0:?} exists already, no --force given")]
    ObjectStoreExists(OsString),

    #[error("{0:?} exists and is not empty")]
    ObjectStoreForeignExists(OsString),

    #[error("{0:?} is not a directory")]
    ObjectStoreNoDir(OsString),

    #[error("Wrong object type: got '{have:?}' expected '{want:?}'")]
    ObjectType {
        have: identifier_kind::ObjectType,
        want: identifier_kind::ObjectType,
    },

    #[error("can't traverse into a parent object")]
    NoParent,

    #[error("object {0:?} not found")]
    ObjectNotFound(OsString),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
