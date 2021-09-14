mod prelude;
mod uberall;
pub use chrono;
pub use clap;
pub use fern;
pub use lazy_static;
pub use libc;
pub use log;
pub use parking_lot;
pub use syslog;
pub use thiserror;
pub use uberall::UberAll;

use clap::{AppSettings, ArgMatches};
use once_cell::sync::OnceCell;

static OPT_DAEMONIZE: OnceCell<bool> = OnceCell::new();

pub fn init_daemonize(matches: &ArgMatches) {
    OPT_DAEMONIZE
        .set(
            !matches.is_present("foreground") && matches.is_present("daemon")
                || !matches.is_present("debug"),
        )
        .unwrap();
}

/// when allowed to daemonize?
pub fn may_daemonize() -> bool {
    *OPT_DAEMONIZE.get().unwrap()
}

/// if allowed , daemonize the process
pub fn maybe_daemonize() {
    if may_daemonize() {
        daemonize::Daemonize::new().start().unwrap_or_else(|_| {
            log::error!("Failed to daemonize");
            std::process::exit(libc::EXIT_FAILURE);
        });
        log::debug!("daemonized");
    } else {
        log::debug!("do not daemonize");
    }
}
