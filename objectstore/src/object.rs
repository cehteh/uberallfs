use std::fs::File; // placeholder for review
pub struct AccessControl {} // placeholder for review

pub enum IdentifierType {
    PrivateMutable,
    PublicMutable(AccessControl),
    PublicImmutable(AccessControl, File),
    AnonymousImmutable(File),
}

pub fn numeric_id(id: &IdentifierType) -> u8 {
    match id {
        IdentifierType::PrivateMutable => 1,
        IdentifierType::PublicMutable(_) => 2,
        IdentifierType::PublicImmutable(_, _) => 3,
        IdentifierType::AnonymousImmutable(_) => 4,
    }
}

pub enum ObjectType {
    Tree,
    Blob,
}

pub struct Object {
    id_type: IdentifierType,
    hash: [u8; 32],
}

impl Object {
    fn create(id_type: IdentifierType, file_type: ObjectType) -> Object {
        Object {
            id_type,
            hash: [0; 32],
        }
    }
}
