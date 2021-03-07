use clap::ArgMatches;
use std::ffi::OsStr;
use std::fs::{self, create_dir_all};
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

#[allow(unused_imports)]
use log::{debug, error, info, trace};

use crate::objectstore::ObjectStore;

macro_rules! return_other_error {
    ($fmt:literal, $($e:expr),*) => {
        return Err(Error::new(ErrorKind::Other, format!($fmt, $($e,)*)));
    };
}

fn valid_objectstore_dir(dir: &Path, force: bool) -> io::Result<()> {
    //PLANNED: can this be integrated in the clap validator?
    // allow init when dir:
    //   - is not a symlink AND
    //     - does not exist
    //     - is an empty directory
    //     - exists AND is already an objectstore AND the --force option was given
    if dir.exists() {
        if dir
            .symlink_metadata().map(|dir_m| dir_m.file_type().is_dir())
            .unwrap_or(false)
        {
            let mut objectstore_dir = PathBuf::from(dir);
            objectstore_dir.push("objectstore.version");

            if objectstore_dir.is_file() {
                if !force {
                    return_other_error!(
                        "'{}' exists, is a objectstore, no --force given",
                        dir.display()
                    )
                }
            } else if dir.read_dir()?.next().is_some() {
                return_other_error!("'{}' exist and is not empty", dir.display())
            }
        } else {
            return_other_error!("'{}' is not a directory", dir.display())
        }
    }

    Ok(())
}


pub(crate) fn init(dir: &OsStr, matches: &ArgMatches) -> io::Result<()> {
    let dir = Path::new(dir);

    valid_objectstore_dir(dir, matches.is_present("force"))?;

    create_dir_all(dir)?;

    // initialize objectstore structure
    let mut objectstore_dir = PathBuf::from(dir);
    objectstore_dir.push("config");
    create_dir_all(&objectstore_dir)?;

    let mut objectstore_dir = PathBuf::from(dir);
    objectstore_dir.push("objects");
    create_dir_all(&objectstore_dir)?;

    for sub in ["tmp", "delete", "volatile"].iter() {
        objectstore_dir.push(sub);
        create_dir_all(&objectstore_dir)?;
        objectstore_dir.pop();
    }

    const URL_SAFE_ENCODE: &[u8; 64] = &*b"ABCDEFGHIJLKMNOPQRSTUVWXYZabcdefghijlkmnopqrstuvwxyz0123456789-_";
    for a in URL_SAFE_ENCODE.iter() {
        for b in URL_SAFE_ENCODE.iter() {
            objectstore_dir.push(format!("{}{}", *a as char, *b as char));
            create_dir_all(&objectstore_dir)?;
            objectstore_dir.pop();
        }
    }

    let mut objectstore_dir = PathBuf::from(dir);
    objectstore_dir.push("objectstore.version");
    fs::write(objectstore_dir, format!("{}\n", crate::VERSION))?;

    let objectstore = ObjectStore::open(dir);

    //TODO: unpack and verify import

    // or create a private new root

    // link the rootdir
    //   - objects/root/ :: symlink to the root dir object


    Ok(())
}
