use crate::prelude::*;

use clap::ArgMatches;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

use fuser::MountOption;
use std::path::Path;
use std::ffi::OsStr;

use objectstore::objectstore::ObjectStore;

use crate::uberallfs::UberallFS;

pub(crate) fn opt_mount(mountpoint: &OsStr, matches: &ArgMatches) -> Result<()> {
    trace!("mountpoint: {:?}", mountpoint);
    let objectstore_dir = matches
        .value_of_os("OBJECTSTORE")
        .or(Some(mountpoint))
        .unwrap();

    trace!("objectstore: {:?}", objectstore_dir);
    let mut objectstore = ObjectStore::open(Path::new(objectstore_dir))?;

    let offline = matches.is_present("offline");

    let mut options = vec![
        MountOption::RO,
        MountOption::FSName("uberallfs".to_string()),
    ];
    options.push(MountOption::AutoUnmount); //TODO: optarg?

    fuser::mount2(UberallFS, mountpoint, &options);

    Ok(())
}
