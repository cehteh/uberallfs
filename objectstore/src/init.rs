use std::fs::create_dir_all;
#[cfg(unix)]
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::io::Write;

use itertools::repeat_n;
use uberall::clap::ArgMatches;
use uberall::UberAll;
use openat_ct as openat;
use openat::Dir;

use crate::prelude::*;
use crate::identifier_kind::*;
use crate::objectstore::{lock_fd, LockingMethod::*, ObjectStore};

fn valid_objectstore_dir(dir: &Path, force: bool) -> Result<Dir> {
    // PLANNED: can this be integrated in the clap validator?
    //   https://github.com/clap-rs/clap/discussions/2387
    // allow init when dir:
    //   - is not a symlink AND
    //     - does not exist
    //     - is an empty directory
    //     - exists AND is already an objectstore AND the --force option was given
    if dir.exists() {
        if dir
            .symlink_metadata()
            .map(|dir_m| dir_m.file_type().is_dir())
            .unwrap_or(false)
        {
            let mut objectstore_dir = PathBuf::from(dir);
            objectstore_dir.push("objects/version");

            if objectstore_dir.is_file() {
                if !force {
                    return Err(ObjectStoreError::ObjectStoreExists(dir.into()).into());
                }
            } else if dir.read_dir()?.next().is_some() {
                return Err(ObjectStoreError::ObjectStoreForeignExists(dir.into()).into());
            }
        } else {
            return Err(ObjectStoreError::ObjectStoreNoDir(dir.into()).into());
        }
    } else {
        create_dir_all(dir)?;
    }

    let base_dir = Dir::flags().open(dir)?;
    lock_fd(&base_dir, TryLock)?;
    Ok(base_dir)
}

pub(crate) fn opt_init(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let force = matches.is_present("force");
    let base_dir = valid_objectstore_dir(dir.as_ref(), force)?;
    debug!(
        "Initialize objectstore in {:?}, version {}",
        dir,
        crate::VERSION
    );

    let objectstore = ObjectStore::create_objectstore(base_dir, force)?;

    use crate::object::Object;
    let maybe_root = if let Some(archive) = matches.value_of_os("ARCHIVE") {
        // imported for its side-effects, even when no-root is given,
        // otherwise defines the new root
        let root = objectstore.import(archive)?;
        if !matches.is_present("noroot") {
            Some(Ok(root))
        } else {
            None
        }
    } else if !matches.is_present("noroot") {
        Some(
            Object::build(
                ObjectType::Directory,
                SharingPolicy::Private,
                Mutability::Mutable,
            )
            .realize(&objectstore),
        )
    } else {
        None
    };

    match maybe_root {
        Some(Ok(root)) => objectstore.set_root(&root.identifier),
        Some(Err(err)) => Err(err),
        None => Ok(()),
    }
}

impl ObjectStore {
    /// Create and initialize a new objectstore at the given dir.
    pub fn create_objectstore(dir: Dir, reinit: bool) -> Result<ObjectStore> {
        let _ = Dir::create_dir(&dir, "objects", 0o770);

        // needs special flags for the directory lock
        let objects = dir
            .with(openat::O_DIRECTORY)
            .without(openat::O_PATH | openat::O_SEARCH)
            .sub_dir("objects")?;
        lock_fd(&objects, TryLock)?;

        let mut version = objects.write_file("version", 0o770)?;
        debug!("creating version: {}", crate::VERSION);
        version.write_all(format!("{}\n", crate::VERSION).as_bytes())?;

        // initialize objectstore structure
        for sub in ["tmp", "delete"] {
            match objects.create_dir(sub, 0o770) {
                Ok(()) => {
                    trace!("creating dir: objects/{}", sub);
                }
                Err(err) if reinit && err.kind() == io::ErrorKind::AlreadyExists => {
                    trace!("reusing dir: objects/{}", sub);
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        }

        // https://github.com/marshallpierce/rust-base64/issues/41
        const URL_SAFE_ENCODE: &[u8; 64] =
            &*b"ABCDEFGHIJLKMNOPQRSTUVWXYZabcdefghijlkmnopqrstuvwxyz0123456789-_";

        URL_SAFE_ENCODE
            .iter()
            .flat_map(|c| repeat_n(c, URL_SAFE_ENCODE.len()))
            .zip(URL_SAFE_ENCODE.iter().cycle())
            .try_for_each(|(a, b)| -> Result<()> {
                // PLANNED: objects/delete/ab/
                let ab = [*a, *b];
                let dirlevel = OsStr::from_bytes(&ab);
                match objects.create_dir(dirlevel, 0o770) {
                    Ok(()) => {
                        trace!("creating dir: objects/{:?}", dirlevel);
                        Ok(())
                    }
                    Err(err) if reinit && err.kind() == io::ErrorKind::AlreadyExists => {
                        trace!("reusing dir: objects/{:?}", dirlevel);
                        Ok(())
                    }
                    Err(err) => Err(err.into()),
                }
            })?;

        Ok(ObjectStore {
            version: crate::VERSION,
            objects,
            uberall: UberAll::new()?,
        })
    }
}
