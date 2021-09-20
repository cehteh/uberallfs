pub mod daemon;
mod prelude;
mod uberall;
pub use chrono;
pub use clap;
pub use fern;
pub use ipc_channel;
pub use lazy_static;
pub use libc;
pub use log;
pub use parking_lot;
pub use serde;
pub use syslog;
pub use thiserror;
pub use uberall::UberAll;

use prelude::*;
use std::error::Error;

pub fn error_to_exitcode(error: Box<dyn Error>) -> i32 {
    error
        .downcast::<io::Error>()
        .map_or(libc::EXIT_FAILURE, |e| {
            e.raw_os_error().unwrap_or(match e.kind() {
                io::ErrorKind::AlreadyExists => libc::EEXIST,
                io::ErrorKind::Other => libc::EXIT_FAILURE,
                _ => todo!("implement for kind {:?}", e.kind()),
            })
        })
}
