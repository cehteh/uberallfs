mod prelude;
mod uberall;
use prelude::*;
pub use chrono;
pub use clap;
pub use fern;
pub use ipc_channel;
pub use lazy_static;
pub use libc;
pub use log;
pub use parking_lot;
pub use syslog;
pub use thiserror;
pub use uberall::UberAll;

use clap::{AppSettings, ArgMatches};
use ipc_channel::ipc;
use once_cell::sync::OnceCell;
use std::error::Error;
use std::ffi::OsString;
use std::process;

static DAEMONIZE: OnceCell<bool> = OnceCell::new();
static PIDFILE: OnceCell<OsString> = OnceCell::new();

pub fn error_to_exitcode(error: Box<dyn Error>) -> i32 {
    error
        .downcast::<io::Error>()
        .map_or(libc::EXIT_FAILURE, |e| {
            e.raw_os_error().unwrap_or(match e.kind() {
                io::ErrorKind::AlreadyExists => libc::EEXIST,
                _ => todo!("implement for kind {:?}", e.kind()),
            })
        })
}

pub fn init_daemonize(matches: &ArgMatches) {
    DAEMONIZE
        .set(
            !matches.is_present("foreground") && matches.is_present("background")
                || !matches.is_present("debug"),
        )
        .unwrap();

    if let Some(pidfile) = matches.value_of_os("pidfile") {
        PIDFILE.set(pidfile.into()).unwrap();
    }
}

/// when allowed to daemonize?
pub fn may_daemonize() -> bool {
    *DAEMONIZE.get().unwrap()
}

/// if allowed , daemonize the process
pub fn maybe_daemonize<
    F: FnOnce(Option<ipc::IpcSender<std::option::Option<i32>>>) -> Result<()> + Copy,
>(
    childfunc: F,
) -> Result<()> {
    if may_daemonize() {
        let mut daemon = daemonize::Daemonize::new().working_directory(".");

        if let Some(pidfile) = PIDFILE.get() {
            daemon = daemon.pid_file(pidfile);
        };

        log::info!("main pid: {}", process::id());

        let (tx, rx) = ipc::channel::<Option<i32>>()?;

        match daemon.execute() {
            daemonize::Outcome::Child(_) => {
                drop(rx);
                match childfunc(Some(tx)) {
                    Ok(()) => {
                        info!("daemon shut down");
                        std::process::exit(libc::EXIT_SUCCESS);
                    }
                    Err(err) => {
                        error!("daemon exited with: {:?}", err);
                        std::process::exit(error_to_exitcode(err));
                    }
                }
            }
            daemonize::Outcome::Parent(_) => {
                drop(tx);
                if let Some(exitcode) = rx.recv().unwrap_or(Some(libc::EPIPE)) {
                    let err = io::Error::from_raw_os_error(exitcode);
                    log::debug!("daemonize error: {:?}", err);
                    Err(err.into())
                } else {
                    log::debug!("daemonized");
                    Ok(())
                }
            }
        }
    } else {
        log::debug!("do not daemonize");
        childfunc(None).into()
    }
}
