use uberall::clap::{self, App, Arg};

pub fn uberallfs_optargs() -> App<'static, 'static> {
    App::new("uberallfs")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("distributed filesystem for the real world")
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .multiple(true)
                .help("Enable debug mode, disables daemonize unless explicitly requested"),
        )
        .arg(
            Arg::with_name("background")
                .long("background")
                .help(
                    "Fork into background, if applicable, enables logging to syslog when no --logfile is given",
                )
        )
        .arg(
            Arg::with_name("foreground")
                .long("foreground")
                .help("Do not fork into background"),
        )
        .arg(
            Arg::with_name("logfile")
                .long("logfile")
                .takes_value(true)
                .value_name("LOGFILE")
                .help("Specify a a filename for logging"),
        )
        .arg(
            Arg::with_name("pidfile")
                .long("pidfile")
                .takes_value(true)
                .value_name("PIDFILE")
                .help("Path to a pidfile when daemonizing"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Suppress any log output"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Increment verbosity level"),
        )
}
