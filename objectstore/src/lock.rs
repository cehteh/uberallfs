use std::ffi::OsStr;

use uberall::clap::ArgMatches;
use uberall::libc;

use crate::prelude::*;
use crate::objectstore::{LockingMethod::*, ObjectStore};

pub(crate) fn opt_lock(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let mut objectstore = ObjectStore::open(
        dir.as_ref(),
        if matches.is_present("wait") {
            WaitForLock
        } else {
            TryLock
        },
    )?;

    // Wait until the process is killed (ctrl-c)
    loop {
        unsafe {
            libc::sleep(1);
        }
    }
}
