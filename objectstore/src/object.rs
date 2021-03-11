use std::ffi::OsString;
use std::fs::File;

/*
Objects come in different flavors:

  ObjectTypes:
    File
    Directory
    (maybe more in future)

  Ownership/Security:
    Private / not shared
    Public / shared with Creator and ACL's
    Anonymous / Shared without restrictions

  Mutability:
    Mutable
    Immutable
 */

pub struct Identifier {
    identifier: IdType,
    id_type: IdEnum,
    base64: Option<OsString>,
}

pub type IdType = [u8; 32];

#[repr(u8)]
enum IdEnum {
    PrivateMutable = 1,
    PublicMutable,
    PublicImmutable,
    AnonymousImmutable,
}

#[repr(u8)]
pub enum ObjectType {
    Tree = 1,
    Blob,
}

pub enum Create {
    PrivateMutable(IdType),
    PublicMutable(IdType, Creator, Acl),
    PublicImmutable(File, Creator, Acl),
    AnonymousImmutable(File),
}

impl Create {
    fn get_idenum(&self) -> IdEnum {
        match self {
            Self::PrivateMutable(_) => IdEnum::PrivateMutable,
            Self::PublicMutable(_, _, _) => IdEnum::PublicMutable,
            Self::PublicImmutable(_, _, _) => IdEnum::PublicImmutable,
            Self::AnonymousImmutable(_) => IdEnum::AnonymousImmutable,
        }
    }
}

pub struct Object {
    identifier: Identifier, // hash key
    object_type: ObjectType,
    object_impl: ObjectImpl,
}

impl Object {
    pub(crate) fn create(object_type: ObjectType, create_args: Create) -> Object {
        let id_type = create_args.get_idenum();
        match create_args {
            Create::PrivateMutable(identifier) => Object {
                identifier: Identifier {
                    identifier,
                    id_type,
                    base64: None,
                },
                object_type,
                object_impl: ObjectImpl::PrivateMutable(PrivateMutableObject),
            },

            Create::PublicMutable(identifier, creator, acl) => Object {
                identifier: Identifier {
                    identifier,
                    id_type,
                    base64: None,
                },
                object_type,
                object_impl: ObjectImpl::PublicMutable(PublicMutableObject { creator, acl }),
            },

            //PublicImmutable(File, Creator, Acl),
            //AnonymousImmutable(File),
            _ => unimplemented!(),
        }
    }
}

enum ObjectImpl {
    PrivateMutable(PrivateMutableObject),
    PublicMutable(PublicMutableObject),
    PublicImmutable(PublicImmutableObject),
    AnonymousImmutable(AnonymousImmutableObject),
}

struct PrivateMutableObject;

struct PublicMutableObject {
    creator: Creator,
    acl: Acl,
}

struct PublicImmutableObject;
struct AnonymousImmutableObject;

pub struct Acl;
pub struct Creator;
