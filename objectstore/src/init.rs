use clap::ArgMatches;
use std::ffi::OsStr;
use std::fs::{self, DirBuilder};
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use log::{debug, error, info, trace};

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
    //     - exists and is already an objectstore AND the --force option was given
    if dir.exists() {
        if dir
            .symlink_metadata()
            .and_then(|dir_m| Ok(dir_m.file_type().is_dir()))
            .unwrap_or(false)
        {
            let mut objectstore_dir = PathBuf::from(dir);
            objectstore_dir.push(".uberallfs.objectstore.version");

            if objectstore_dir.is_file() && !force {
                return_other_error!(
                    "'{}' exists, is a objectstore, no --force given",
                    dir.display()
                )
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
    valid_objectstore_dir(Path::new(dir), matches.is_present("force"))?;

    //    DirBuilder::new().recursive(true).create(dir)?;

    // initialize objectstore structure

    // unpack and verify import

    // or create a private new root

    // link the rootdir

    Ok(())
}
