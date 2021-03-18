use crate::prelude::*;

use std::ffi::OsString;
use std::fs::File;

use crate::objectstore::{
    DirectoryPermissions,
    FileAttributes,
    FilePermissions,
    ObjectStore,
};
//use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct Acl;
#[derive(Debug)]
pub struct Creator;

use crate::identifier::{Identifier, IdentifierBin};
use crate::identifier_kind::*;

pub struct Object {
    pub identifier: Identifier,
    opts: ObjectImpl,
}

impl Object {
    pub fn new(
        object_type: ObjectType,
        sharing_policy: SharingPolicy,
        mutability: Mutability,
        binary: IdentifierBin,
    ) -> Self {
        let kind = IdentifierKind::create(object_type, sharing_policy, mutability);
        Object {
            identifier: Identifier::from_binary(kind, binary),
            opts: ObjectImpl::new(kind),
        }
    }

    pub fn realize(mut self, objectstore: &ObjectStore) -> Result<Object> {
        self.opts
            .realize(&self.identifier, objectstore)
            .and(Ok(self))
    }
}

#[derive(Debug)]
enum ObjectImpl {
    Unimplemented,
    PrivateMutable,
    PublicImmutableFile {
        creator: Option<Creator>,
        acl: Option<Acl>,
        //from file
    },
}

impl ObjectImpl {
    fn new(kind: IdentifierKind) -> ObjectImpl {
        use crate::identifier_kind::{Mutability::*, ObjectType::*, SharingPolicy::*};
        match kind.components() {
            (_, Private, Mutable) => ObjectImpl::PrivateMutable,
            (File, PublicAcl, Immutable) => ObjectImpl::PublicImmutableFile {
                creator: None,
                acl: None,
            },
            _ => ObjectImpl::Unimplemented,
        }
    }

    fn realize(&self, identifier: &Identifier, objectstore: &ObjectStore) -> Result<()> {
        match self {
            ObjectImpl::PrivateMutable => {
                objectstore.create_directory(identifier, None, DirectoryPermissions::new().full())
            }

            _ => Err(Error::from(ObjectStoreError::UnsupportedObjectType)),
        }
    }
}
