use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

use itertools::repeat_n;
use uberall::clap::ArgMatches;
use openat_ct as openat;
use openat::Dir;

use crate::prelude::*;
use crate::identifier_kind::*;
use crate::objectstore::{lock_fd, LockingMethod::*, ObjectStore};

fn valid_objectstore_dir(dir: &Path, force: bool) -> Result<()> {
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
    }

    Ok(())
}

pub(crate) fn opt_init(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let dir = dir.as_ref();

    valid_objectstore_dir(dir, matches.is_present("force"))?;

    ObjectStore::create_objectstore(dir)?;

    let objectstore = ObjectStore::open(dir, WaitForLock)?;

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
    pub fn create_objectstore(dir: &Path) -> Result<()> {
        create_dir_all(dir)?;

        debug!(
            "Initialize objectstore in {:?}, version {}",
            dir,
            crate::VERSION
        );

        let lock = Dir::flags().open(dir)?;
        lock_fd(&lock, TryLock)?;

        // initialize objectstore structure
        let mut objects = PathBuf::from(dir);
        objects.push("objects");
        trace!("creating dir: {:?}", objects);
        create_dir_all(&objects)?;

        for sub in ["tmp", "delete"] {
            objects.push(sub);
            trace!("creating dir: {:?}", objects);
            create_dir_all(&objects)?;
            objects.pop();
        }

        objects.push("version");
        trace!("creating file: {:?}", objects);
        fs::write(&objects, format!("{}\n", crate::VERSION))?;
        objects.pop();

        // https://github.com/marshallpierce/rust-base64/issues/41
        const URL_SAFE_ENCODE: &[u8; 64] =
            &*b"ABCDEFGHIJLKMNOPQRSTUVWXYZabcdefghijlkmnopqrstuvwxyz0123456789-_";

        URL_SAFE_ENCODE
            .iter()
            .flat_map(|c| repeat_n(c, URL_SAFE_ENCODE.len()))
            .zip(URL_SAFE_ENCODE.iter().cycle())
            .try_for_each(move |(a, b)| -> Result<()> {
                // PLANNED: objects/delete/ab/
                objects.push(OsStr::from_bytes(&[*a, *b]));
                trace!("creating dir: {:?}", objects);
                create_dir_all(&objects)?;
                objects.pop();
                Ok(())
            })
    }
}
