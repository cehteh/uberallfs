use crate::prelude::*;

use clap::ArgMatches;
use std::path::PathBuf;

use std::ffi::OsStr;

use crate::identifier_kind::*;
use crate::object::Object;
use crate::objectstore::{ObjectStore, SubObject};

pub(crate) fn opt_mkdir(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let mut objectstore = ObjectStore::open(dir.as_ref())?;

    let mut sharing_policy = SharingPolicy::Private;

    let acl = if let Some(_acls) = matches.value_of("acl") {
        sharing_policy = SharingPolicy::PublicAcl;
        Some(crate::object::Acl {})
    } else {
        None
    };

    let (src, remaining) = objectstore.path_lookup(
        &matches
            .value_of_os("PATH")
            .map(PathBuf::from).unwrap(),
        None,
    )?;
    src.ensure_dir()?;

    if !remaining.as_os_str().is_empty() {
        trace!("mkdir src: {:?}/{:?}", src, remaining.as_os_str());

        if remaining.components().next() == None {
            return Err(ObjectStoreError::ObjectExists.into());
        }
        assert_eq!(
            remaining.components().count(),
            1,
            "TODO: parent dir handling"
        );

        let object = match matches.value_of("SOURCE") {
            Some(_base64) => {
                if acl.is_some() {
                    return Err(ObjectStoreError::OptArgError(String::from(
                        "ACL can only be used with new objects",
                    ))
                    .into());
                };
                unimplemented!("use existing object")
            }

            None => Object::new(
                ObjectType::Directory,
                sharing_policy,
                Mutability::Mutable,
                objectstore.rng_identifier(),
            )
            .acl(acl)
            .realize(&objectstore)?,
        };

        trace!("mkdir dest: {:?}", &object.identifier);

        //FIXME: remove object when failed and not from SOURCE
        objectstore.create_link(&object.identifier, SubObject(&src, remaining.as_os_str()))?;
    } else {
        return Err(ObjectStoreError::OptArgError(String::from("PATH foo")).into());
    }

    Ok(())
}
