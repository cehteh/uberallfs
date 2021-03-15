use std::ffi::OsString;
use std::fs::File;
use std::io;

use crate::objectstore::ObjectStore;
//use serde::{Serialize, Deserialize};

use crate::identifier::{self, Identifier, IdentifierBin};
use crate::identifier_kind::{Mutability::*, ObjectType::*, SharingPolicy::*, *};

pub(crate) enum Create {
    PrivateMutable(identifier::IdentifierBin),
    PublicMutable(identifier::IdentifierBin, Creator, Acl),
    PublicImmutable(File, Creator, Acl),
    AnonymousImmutable(File),
}

impl Create {
    fn get_kind(&self, object_type: ObjectType) -> IdentifierKind {
        match self {
            Self::PrivateMutable(_) => IdentifierKind::create(object_type, Private, Mutable),
            Self::PublicMutable(_, _, _) => IdentifierKind::create(object_type, PublicAcl, Mutable),
            Self::PublicImmutable(_, _, _) => {
                IdentifierKind::create(object_type, PublicAcl, Immutable)
            }
            Self::AnonymousImmutable(_) => {
                IdentifierKind::create(object_type, Anonymous, Immutable)
            }
        }
    }
}

pub(crate) struct Object {
    identifier: Identifier,
    object_impl: ObjectImpl,
}

impl Object {
    pub(crate) fn create(object_type: ObjectType, create_args: Create) -> Object {
        let kind = create_args.get_kind(object_type);
        match create_args {
            Create::PrivateMutable(identifier) => {
                PrivateMutableObject::create(Identifier::from_binary(kind, identifier))
            }

            /*
                        Create::PublicMutable(identifier, creator, acl) => Object {
                            identifier: Identifier {
                                identifier,
                                id_type,
                                base64: None,
                            },
                            object_type,
                            object_impl: ObjectImpl::PublicMutable(PublicMutableObject { creator, acl }),
                        },
            */
            //PublicImmutable(File, Creator, Acl),
            //AnonymousImmutable(File),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn realize(self, _objectstore: &ObjectStore) -> io::Result<Object> {
        println!(
            "base64: {}",
            std::str::from_utf8(&self.identifier.id_base64().0).unwrap()
        );
        Ok(self)
    }
}

struct PrivateMutableObject;

impl PrivateMutableObject {
    fn create(identifier: Identifier) -> Object {
        Object {
            identifier,
            object_impl: ObjectImpl::PrivateMutable(PrivateMutableObject),
        }
    }
}

struct PublicMutableObject {
    creator: Creator,
    acl: Acl,
}

struct PublicImmutableObject;
struct AnonymousImmutableObject;

enum ObjectImpl {
    PrivateMutable(PrivateMutableObject),
    PublicMutable(PublicMutableObject),
    PublicImmutable(PublicImmutableObject),
    AnonymousImmutable(AnonymousImmutableObject),
}

pub(crate) struct Acl;
pub(crate) struct Creator;
