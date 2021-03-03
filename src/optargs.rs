use clap::{App, Arg};

pub fn uberallfs_optargs() -> App<'static, 'static> {
    App::new("uberallfs")
        .version(crate_version!())
        .author(crate_authors!())
        .about("distributed filesystem for the real world")
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Enable debug mode"),
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
