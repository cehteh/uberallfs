use crate::prelude::*;

use clap::ArgMatches;

use fuser::MountOption;
use std::path::Path;
use std::ffi::OsStr;

use objectstore::ObjectStore;

use crate::uberallfs::UberallFS;

pub(crate) fn opt_mount(mountpoint: &OsStr, matches: &ArgMatches) -> Result<()> {
    trace!("mountpoint: {:?}", mountpoint);
    let objectstore_dir = Path::new(
        matches
            .value_of_os("OBJECTSTORE")
            .or(Some(mountpoint))
            .unwrap(),
    );

    let mountpoint = Path::new(mountpoint);

    trace!("objectstore: {:?}", objectstore_dir);

    UberallFS::new(objectstore_dir)?.mount(
        mountpoint,
        matches.is_present("offline"),
        matches.value_of_os("root"),
        None,
    )
}
