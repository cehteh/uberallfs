use crate::prelude::*;

use clap::ArgMatches;

use fuser::MountOption;
use std::ffi::OsStr;

use crate::uberallfs::UberallFS;

pub(crate) fn opt_mount(mountpoint: &OsStr, matches: &ArgMatches) -> Result<()> {
    trace!("mountpoint: {:?}", mountpoint);
    let objectstore_dir = matches
        .value_of_os("OBJECTSTORE")
        .or(Some(mountpoint))
        .unwrap()
        .as_ref();

    let mountpoint = mountpoint.as_ref();

    trace!("objectstore: {:?}", objectstore_dir);

    uberall::maybe_daemonize();

    UberallFS::new(objectstore_dir)?.mount(
        mountpoint,
        matches.is_present("offline"),
        matches.value_of_os("root").unwrap_or_default(),
        None,
    )
}
