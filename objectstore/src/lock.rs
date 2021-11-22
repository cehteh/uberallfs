use std::ffi::OsStr;

use uberall::clap::ArgMatches;
use uberall::libc;

use crate::prelude::*;
use crate::objectstore::ObjectStore;

pub(crate) fn opt_lock(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let _objectstore = ObjectStore::open(
        dir.as_ref(),
        if matches.is_present("wait") {
            LockingMethod::WaitForLock
        } else {
            LockingMethod::TryLock
        },
    )?;

    // Wait until the process is killed (ctrl-c)
    loop {
        unsafe {
            libc::sleep(1);
        }
    }
}

/// Opening an objectstore will lock its directory, to obtain this lock there are two methods.
///
///  * TryLock:: Try to lock the objectstore and return an error immediately when that fails.
///  * WaitForLock:: Wait until the lock becomes available.
#[derive(PartialEq)]
pub enum LockingMethod {
    TryLock,
    WaitForLock,
}

/// Place an exclusive lock on a file descriptor
/// This lock will exist as long the file descriptor is open.
#[cfg(unix)]
pub fn lock_fd<T: std::os::unix::io::AsRawFd>(fd: &T, locking_method: LockingMethod) -> Result<()> {
    let mut lockerr;

    // first try locking without wait
    loop {
        lockerr = unsafe {
            if libc::flock(fd.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) == -1 {
                *libc::__errno_location()
            } else {
                0
            }
        };
        if lockerr != libc::EINTR {
            break;
        };
    }

    if lockerr == libc::EWOULDBLOCK {
        // when that failed and waiting was requested we now wait for the lock
        if locking_method == LockingMethod::WaitForLock {
            warn!("Waiting for lock");
            loop {
                lockerr = unsafe {
                    if libc::flock(fd.as_raw_fd(), libc::LOCK_EX) == -1 {
                        *libc::__errno_location()
                    } else {
                        0
                    }
                };
                if lockerr != libc::EINTR {
                    break;
                };
            }
        } else {
            return Err(ObjectStoreError::NoLock.into());
        }
    };

    if lockerr != 0 {
        let err = io::Error::from_raw_os_error(lockerr);
        trace!("objectstore locking error: {:?}", err);
        Err(err.into())
    } else {
        info!("objectstore locked");
        Ok(())
    }
}
