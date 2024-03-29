use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use uberall::clap::ArgMatches;

use crate::prelude::*;
use crate::identifier_kind::*;
use crate::object::Object;
use crate::{
    DirectoryPermissions, Identifier, LockingMethod::*, ObjectPath, ObjectStore, SubObject,
};

pub(crate) fn opt_mkdir(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let objectstore = ObjectStore::open(dir.as_ref(), WaitForLock)?;

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
            return Err(ObjectStoreError::ObjectExists(
                matches.value_of_os("PATH").unwrap().into(),
            )
            .into());
        }

        let count = remaining.components().count();

        // create parent dirs
        if count > 1 {
            if matches.is_present("parents") {
                for name in remaining.components().take(count - 1) {
                    let name = name.as_os_str();
                    info!("create: {:?}", name);

                    let object =
                        Object::build(ObjectType::Directory, sharing_policy, Mutability::Mutable)
                            .acl(&acl)
                            .realize(&objectstore)?;
                    trace!("identifier: {:?}", &object.identifier);

                    objectstore.create_link(&object.identifier, SubObject(&src, name))?;

                    src = object.identifier;
                }
            } else {
                let name = remaining.components().next().unwrap().as_os_str().into();
                warn!("Parent dir missing, no -p given: {:?}", &name);
                return Err(ObjectStoreError::ObjectNotFound(name).into());
            }
        }

        let object = match matches.value_of_os("SOURCE") {
            Some(path) => {
                if acl.is_some() {
                    return Err(ObjectStoreError::OptArgError(String::from(
                        "ACL can only be used with new objects",
                    ))
                    .into());
                };

                let (source_id, empty) = objectstore.path_lookup(Path::new(path), None)?;

                if empty.as_os_str().is_empty() {
                    trace!("found source identifier: {:?}", source_id);
                    source_id.ensure_dir()?;
                } else {
                    warn!("source not found: {:?}", path);
                    return Err(ObjectStoreError::ObjectNotFound(path.into()).into());
                }

                Object::from(source_id)
            }

            None => Object::build(ObjectType::Directory, sharing_policy, Mutability::Mutable)
                .acl(&acl)
                .realize(&objectstore)?,
        };

        trace!("identifier: {:?}", &object.identifier);
        objectstore
            .create_link(
                &object.identifier,
                SubObject(&src, remaining.components().last().unwrap().as_os_str()),
            )
            .map_err(|err| err)
    } else {
        Err(io::Error::from(io::ErrorKind::AlreadyExists).into())
    }
}

impl ObjectStore {
    /// Creates a directory for an 'identifier'.
    pub(crate) fn create_directory(
        &self,
        identifier: &Identifier,
        perm: DirectoryPermissions,
    ) -> Result<()> {
        identifier.ensure_dir()?;
        let mut path = PathBuf::new();
        path.push_identifier(identifier);
        info!("create_directory: {:?}", path.as_os_str());

        self.objects.create_dir(path.as_os_str(), perm.get())?;
        Ok(())
    }
}
