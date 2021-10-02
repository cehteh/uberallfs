use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use uberall::{
    chrono,
    clap::{AppSettings, ArgMatches},
    fern, libc, log, syslog,
};

pub(crate) fn init_logging(matches: &ArgMatches) {
    let mut verbosity_level = 1;

    if matches.is_present("quiet") {
        verbosity_level = 0;
    }

    // allow -dd for 'trace' level
    match matches.occurrences_of("debug") {
        0 => {}
        1 => verbosity_level = 4,
        _ => verbosity_level = 5,
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

    let mut logger = fern::Dispatch::new();

    if matches.is_present("debug") {
        logger = logger.format(move |out, message, record| {
            let thread_id = std::thread::current();
            out.finish(format_args!(
                "{:0>12}: {:>5}: {}:{}: {}: {}",
                seq_num(),
                colors.color(record.level()),
                record.file().unwrap_or(""),
                record.line().unwrap_or(0),
                thread_id.name().unwrap_or("UNKNOWN"),
                message
            ))
        });
    } else {
        logger = logger.format(move |out, message, record| {
            let thread_id = std::thread::current();
            out.finish(format_args!(
                "{:>5}: {}",
                colors.color(record.level()),
                message
            ))
        });
    }

    // Always log to stderr, we may not dameonize
    logger = logger.level(verbosity_level).chain(std::io::stderr());

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

    std::panic::set_hook(Box::new(|i| {
        log::error!(
            "panic at: {}:{}:",
            i.location().map_or("", |l| l.file()),
            i.location().map_or(0, |l| l.line())
        );
    }));

    log::info!("START: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
}
