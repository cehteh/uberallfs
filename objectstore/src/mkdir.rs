use crate::prelude::*;

use clap::ArgMatches;
use std::path::Path;

use std::ffi::OsStr;

use crate::identifier_kind::*;
use crate::object::Object;
use crate::objectstore::{ObjectStore, SubObject};
use crate::opath::OPath;

pub(crate) fn opt_mkdir(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let dir = Path::new(dir);
    let mut objectstore = ObjectStore::open(dir)?;

    let mut sharing_policy = SharingPolicy::Private;

    let acl = if let Some(acls) = matches.value_of("acl") {
        sharing_policy = SharingPolicy::PublicAcl;
        Some(crate::object::Acl {})
    } else {
        None
    };

    let (src, remaining) = objectstore.path_lookup(
        matches
            .value_of_os("PATH")
            .and_then(|f| Some(OPath::prefix(f))),
        None,
    )?;
    src.ensure_dir()?;

    if let Some(names) = remaining {
        trace!("mkdir src: {:?}/{:?}", src, names.as_os_str());

        if names.components().next() == None {
            bail!(ObjectStoreError::ObjectExists)
        }
        assert_eq!(names.components().count(), 1, "TODO: parent dir handling");

        let object = match matches.value_of("SOURCE") {
            Some(base64) => {
                if let Some(_) = acl {
                    bail!(ObjectStoreError::OptArgError(String::from(
                        "ACL can only be used with new objects"
                    )));
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
        objectstore.create_link(&object.identifier, SubObject(&src, names.as_os_str()))?;
    } else {
        bail!(ObjectStoreError::OptArgError(String::from("PATH foo")))
    }

    Ok(())
}
