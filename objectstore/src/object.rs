use crate::prelude::*;

use crate::objectstore::{DirectoryPermissions, ObjectStore};
//use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct Acl;
#[derive(Debug)]
pub struct Creator;

use crate::identifier::{Identifier, IdentifierBuilder};
use crate::identifier_kind::*;

/// An Objectstore object
pub struct Object {
    pub identifier: Identifier,
    opts: ObjectImpl,
}

/// Builder Type for incomplete Objects
pub struct ObjectBuilder {
    identifier: IdentifierBuilder,
    opts: ObjectImpl,
}

impl Object {
    /// Start building an Object with the specified parameters
    #[must_use = "configure the builder and finally call realize()"]
    pub fn build(
        object_type: ObjectType,
        sharing_policy: SharingPolicy,
        mutability: Mutability,
    ) -> ObjectBuilder {
        let kind = IdentifierKind::create(object_type, sharing_policy, mutability);
        ObjectBuilder {
            identifier: Identifier::build(kind),
            opts: ObjectImpl::new(kind),
        }
    }
}

impl ObjectBuilder {
    /// May attach an acl to an object
    //TODO: multiple acl's then not option but conditional incremental
    #[must_use = "configure the builder and finally call realize()"]
    pub fn acl(self, acl: &Option<Acl>) -> Self {
        match acl {
            Some(acl) => todo!(),
            None => {}
        }
        self
    }

    /// Realizes the final Object. This creates the respective files in the backing
    /// 'Objectstore'.
    pub fn realize(self, objectstore: &mut ObjectStore) -> Result<Object> {
        self.opts.realize(self.identifier, objectstore)
    }
}

/// Implements the diffent kinds of objects. Implementation detail.
#[derive(Debug)]
enum ObjectImpl {
    NotSupported,
    PrivateMutable,
    PublicImmutableFile {
        creator: Option<Creator>,
        acl: Option<Acl>,
        //from file
    },
}

impl ObjectImpl {
    /// Create the ObjectImpl for the given IdentifierKind.
    fn new(kind: IdentifierKind) -> ObjectImpl {
        use crate::identifier_kind::{Mutability::*, ObjectType::*, SharingPolicy::*};
        match kind.components() {
            (_, Private, Mutable) => ObjectImpl::PrivateMutable,
            (File, PublicAcl, Immutable) => ObjectImpl::PublicImmutableFile {
                creator: None,
                acl: None,
            },
            _ => ObjectImpl::NotSupported,
        }
    }

    /// The actual per-ObjectImpl creation on the backing ObjectStore.
    fn realize(
        self,
        identifier: IdentifierBuilder,
        objectstore: &mut ObjectStore,
    ) -> Result<Object> {
        match self {
            ObjectImpl::PrivateMutable => {
                let identifier = identifier.with_binary(objectstore.rng_identifier());
                objectstore.create_directory(&identifier, DirectoryPermissions::new().full())?;

                Ok(Object {
                    identifier,
                    opts: self,
                })
            }

            ObjectImpl::PublicImmutableFile { .. } => {
                todo!();
            }

            ObjectImpl::NotSupported => Err(ObjectStoreError::UnsupportedObjectType.into()),
        }
    }
}
