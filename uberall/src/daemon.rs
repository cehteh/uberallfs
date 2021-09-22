use crate::prelude::*;
use crate::serde::{Deserialize, Serialize};
use ipc_channel::ipc;
use once_cell::sync::OnceCell;

use std::ffi::OsString;
use std::process;

use clap::{AppSettings, ArgMatches};

static PIDFILE: OnceCell<OsString> = OnceCell::new();
static DAEMONIZE: OnceCell<bool> = OnceCell::new();

/// process commandline args: allow to fork in the background when explicitly requested or
/// debugging is not enabled.
pub fn init_daemonize(matches: &ArgMatches) {
    DAEMONIZE
        .set(
            !matches.is_present("foreground")
                && (matches.is_present("background") || !matches.is_present("debug")),
        )
        .unwrap();

    if let Some(pidfile) = matches.value_of_os("pidfile") {
        PIDFILE.set(pidfile.into()).unwrap();
    }

    trace!("may_daemonize: {}", may_daemonize());
}

/// allowed to daemonize?
pub fn may_daemonize() -> bool {
    *DAEMONIZE.get().unwrap()
}

/// daemonize the process, when allowed
pub fn maybe_daemonize<F: FnOnce(Option<ipc::IpcSender<CallbackMessage>>) -> Result<()> + Copy>(
    childfunc: F,
) -> Result<()> {
    if may_daemonize() {
        let mut daemon = daemonize::Daemonize::new().working_directory(".");

        if let Some(pidfile) = PIDFILE.get() {
            daemon = daemon.pid_file(pidfile);
        };

        log::info!("main pid: {}", process::id());

        let (tx, rx) = ipc::channel::<CallbackMessage>()?;

        match daemon.execute() {
            daemonize::Outcome::Child(_) => {
                // close parent side fd
                drop(rx);
                match childfunc(Some(tx)) {
                    Ok(()) => {
                        info!("daemon shut down");
                        std::process::exit(libc::EXIT_SUCCESS);
                    }
                    Err(err) => {
                        error!("daemon exited with: {:?}", err);
                        std::process::exit(crate::error_to_exitcode(err));
                    }
                }
            }
            daemonize::Outcome::Parent(_) => {
                // close child side fd
                drop(tx);
                match rx.recv().unwrap_or(CallbackMessage::faulty()) {
                    CallbackMessage {
                        is_error: true,
                        os_error: Some(err),
                        message: msg,
                    } => {
                        log::debug!("daemonize io error: {:?} {}", err, msg);
                        Err(io::Error::from_raw_os_error(err).into())
                    }
                    CallbackMessage {
                        is_error: true,
                        os_error: None,
                        message: msg,
                    } => {
                        log::debug!("daemonize other error: {}", msg);
                        Err(io::Error::new(io::ErrorKind::Other, msg).into())
                    }
                    _ => {
                        log::debug!("daemonized");
                        Ok(())
                    }
                }
            }
        }
    } else {
        log::debug!("do not daemonize");
        childfunc(None).into()
    }
}

pub type CallbackTx = ipc::IpcSender<CallbackMessage>;

/// The callback state, may hold the callback function and the sending side of the IPC channel
/// to the parent.
pub struct Callback {
    callback: Option<Box<dyn FnOnce(CallbackTx, CallbackMessage)>>,
    tx: Option<CallbackTx>,
}

impl Callback {
    pub fn new() -> Self {
        Callback {
            callback: None,
            tx: None,
        }
    }

    pub fn set(
        &mut self,
        callback: Box<dyn FnOnce(CallbackTx, CallbackMessage)>,
        tx: Option<CallbackTx>,
    ) -> &Self {
        self.callback = Some(callback);
        self.tx = tx;
        self
    }

    pub fn callback_once(&mut self, message: CallbackMessage) {
        if let (Some(callback), Some(tx)) = (self.callback.take(), self.tx.take()) {
            trace!("callback");
            callback(tx, message);
        } else {
            trace!("no callback");
        }
    }

    pub fn is_some(&self) -> bool {
        self.callback.is_some()
    }
}

/// Carry a possible error from the child process to the parent
#[derive(Serialize, Deserialize, Debug)]
pub struct CallbackMessage {
    is_error: bool,
    os_error: Option<i32>,
    message: String,
}

impl CallbackMessage {
    /// from actual io error
    pub fn from_io_error(error: &io::Error) -> Self {
        Self {
            is_error: true,
            os_error: error.raw_os_error(),
            message: error.to_string(),
        }
    }

    /// fallback when no message was received
    pub fn faulty() -> Self {
        Self {
            is_error: true,
            os_error: None,
            message: "No reply from child".to_string(),
        }
    }

    /// No error case
    pub fn success() -> Self {
        Self {
            is_error: false,
            os_error: None,
            message: "".to_string(),
        }
    }
}
