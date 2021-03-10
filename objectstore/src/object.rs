use std::ffi::OsString;
use std::io;

#[repr(u8)]
pub enum IdentifierType {
    PrivateMutable = 1,
    PublicMutable,
    PublicImmutable,
    AnonymousImmutable,
}

pub enum ObjectType {
    Tree = 1,
    Blob,
}

pub struct Identifier {
    hash: [u8; 32],
    id_type: IdentifierType,
}

pub struct Object {
    identifier: Identifier,     // remove from here, let objectstore keep a weak hashmap<Identifier, Object>
    object_type: ObjectType,
    base64: Option<OsString>,
}

impl Object {
    pub fn create_private_mutable(object_type: ObjectType, random: [u8; 32]) -> io::Result<Object> {
        Ok(Object {
            identifier: Identifier {
                hash: random,
                id_type: IdentifierType::PrivateMutable,
            },
            object_type,
            base64: None,
        })
    }
}
