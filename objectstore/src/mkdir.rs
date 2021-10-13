use std::path::PathBuf;
use std::ffi::OsStr;

use uberall::clap::ArgMatches;

use crate::prelude::*;
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

    let (mut src, remaining) = objectstore.path_lookup(
        &matches.value_of_os("PATH").map(PathBuf::from).unwrap(),
        None,
    )?;
    src.ensure_dir()?;

    if !remaining.as_os_str().is_empty() {
        debug!("mkdir: {:?} / {:?}", src.as_os_str(), remaining.as_os_str());

        if remaining.components().next() == None {
            return Err(ObjectStoreError::ObjectExists.into());
        }

        let count = remaining.components().count();
        // create parent dirs
        if count > 1 {
            // TODO: factor out to handle errors and delete subdirs on rollback
            if matches.is_present("parents") {
                for name in remaining.components().take(count - 1) {
                    let name = name.as_os_str();
                    info!("create: {:?}", name);

                    let object =
                        Object::build(ObjectType::Directory, sharing_policy, Mutability::Mutable)
                            .acl(&acl)
                            .realize(&mut objectstore)?;
                    trace!("identifier: {:?}", &object.identifier);

                    objectstore.create_link(&object.identifier, SubObject(&src, name))?;

                    src = object.identifier;
                }
            } else {
                warn!(
                    "Parent dir missing, no -p given: {:?}",
                    remaining.components().next().unwrap().as_os_str()
                );
                return Err(ObjectStoreError::NoParent.into());
            }
        }

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

            None => Object::build(ObjectType::Directory, sharing_policy, Mutability::Mutable)
                .acl(&acl)
                .realize(&mut objectstore)?,
        };

        trace!("identifier: {:?}", &object.identifier);

        // FIXME: remove object when failed and not from SOURCE, remove created parents
        // as well
        objectstore.create_link(
            &object.identifier,
            SubObject(&src, remaining.components().last().unwrap().as_os_str()),
        )
    } else {
        Err(io::Error::from(io::ErrorKind::AlreadyExists).into())
    }
}
