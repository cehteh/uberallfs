use uberall::{
    clap::{AppSettings, ArgMatches},
    fern, libc, log, syslog,
};

mod optargs;
pub use self::optargs::uberallfs_optargs;
mod logging;

fn main() {
    platform_init();
    let matches = uberallfs_optargs()
        .setting(AppSettings::SubcommandRequired)
        .subcommand(objectstore::optargs())
        .subcommand(fuse::optargs())
        .get_matches();

    uberall::daemon::init_daemonize(&matches);
    logging::init_logging(&matches);

    if let Err(err) = match matches.subcommand() {
        ("objectstore", Some(sub_m)) => objectstore::cmd(sub_m),
        ("fuse", Some(sub_m)) => fuse::cmd(sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    } {
        log::error!("{}", &err);
        std::process::exit(uberall::error_to_exitcode(err));
    } else {
        log::info!("OK");
    }
}

#[cfg(unix)]
fn platform_init() {
    unsafe {
        // no 'other' access
        libc::umask(libc::S_IRWXO);
    }
}

