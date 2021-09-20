use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use uberall::{
    chrono,
    clap::{AppSettings, ArgMatches},
    fern, libc, log, syslog,
};

mod optargs;
pub use self::optargs::uberallfs_optargs;

fn main() {
    platform_init();
    let matches = uberallfs_optargs()
        .setting(AppSettings::SubcommandRequired)
        .subcommand(objectstore::optargs())
        .subcommand(fuse::optargs())
        .get_matches();

    uberall::daemon::init_daemonize(&matches);

    init_logging(&matches);

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

fn init_logging(matches: &ArgMatches) {
    let mut verbosity_level = 1;

    if matches.is_present("quiet") {
        verbosity_level = 0;
    }

    // allow -dd for 'trace' level
    match matches.occurrences_of("debug") {
        1 => verbosity_level = 4,
        2 => verbosity_level = 5,
        _ => {}
    }

    verbosity_level += matches.occurrences_of("verbose");

    use log::LevelFilter::*;
    let verbosity_level = match verbosity_level {
        0 => Off,
        1 => Error,
        2 => Warn,
        3 => Info,
        4 => Debug,
        _ => Trace,
    };

    use uberall::fern::colors::Color::*;
    let colors = fern::colors::ColoredLevelConfig::new()
        .error(Red)
        .warn(Yellow)
        .info(Green)
        .debug(Black)
        .trace(BrightBlack);

    let counter: AtomicU64 = AtomicU64::new(0);

    let seq_num = move || counter.fetch_add(1, Ordering::SeqCst);

    let mut logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            let thread_id = std::thread::current();
            out.finish(format_args!(
                "{:0>16}: {:>5}: {}: {}: {}",
                seq_num(),
                colors.color(record.level()),
                thread_id.name().unwrap_or("UNKNOWN"),
                record.target(),
                message
            ))
        })
        .level(verbosity_level)
        // Always log to stderr, we may not dameonize
        .chain(std::io::stderr());

    if uberall::daemon::may_daemonize() {
        let syslog_formatter = syslog::Formatter3164 {
            facility: syslog::Facility::LOG_USER,
            hostname: None,
            process: "uberallfs".to_owned(),
            pid: 0,
        };
        logger = logger.chain(syslog::unix(syslog_formatter).expect("syslog opened"));
    }

    if let Some(logfile) = matches.value_of_os("logfile") {
        logger = logger.chain(fern::log_file(logfile).expect("opening logfile ok"));
    }

    logger.apply().expect("initialized the logging system");

    log::info!("START: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
}
