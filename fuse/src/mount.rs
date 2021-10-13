use std::ffi::{OsStr, OsString};

use uberall::{
    addy::{self, Signal::*},
    clap::{self, ArgMatches},
};
use fuser::MountOption;

use crate::prelude::*;
use crate::uberallfs::UberallFS;

pub(crate) fn opt_mount(mountpoint: &OsStr, matches: &ArgMatches) -> Result<()> {
    trace!("mountpoint: {:?}", mountpoint);
    let objectstore_dir = matches
        .value_of_os("OBJECTSTORE")
        .or(Some(mountpoint))
        .unwrap()
        .as_ref();

    trace!("objectstore: {:?}", objectstore_dir);

    uberall::daemon::maybe_daemonize(|tx| {
        let umountpoint = OsString::from(mountpoint);
        addy::mediate(SIGINT)
            .register("unmount", move |_signal| {
                std::process::Command::new("fusermount")
                    .arg("-u")
                    .arg(&umountpoint)
                    .status()
                    .expect("umount");
            })?
            .enable()?;

        UberallFS::new(objectstore_dir)?
            .with_callback(
                |tx, m| {
                    debug!("callback called");
                    tx.send(m).expect("send message");
                },
                tx,
            )
            .mount(
                mountpoint.as_ref(),
                matches.is_present("offline"),
                matches.value_of_os("root").unwrap_or_default(),
                None,
            )
    })
}
