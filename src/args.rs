use clap::{App, Arg};

pub fn uberallfs_args() -> App<'static, 'static> {
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
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .long("verbose")
                .help("Increment verbosity level"),
        )
}
