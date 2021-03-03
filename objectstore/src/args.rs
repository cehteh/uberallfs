use clap::{App, AppSettings, Arg, SubCommand};

pub fn args() -> App<'static, 'static> {
    SubCommand::with_name("objectstore")
        .about("Objectstore management")
        .arg(
            Arg::with_name("DIRECTORY")
                .required(true)
                .help("The objectstore directory"),
        )
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize a new objectstore")
                .arg(
                    Arg::with_name("force")
                        .short("f")
                        .long("force")
                        .help("Force overwriting an existing directory"),
                )
                .arg(
                    Arg::with_name("import")
                        .short("i")
                        .long("import")
                        .takes_value(true)
                        .help("Import root directory"),
                ),
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("Exports an object")
                .arg(
                    Arg::with_name("ID_OR_PATH")
                        .required(true)
                        .help("The object to export"),
                )
                //TODO:  glob type
                .arg(
                    Arg::with_name("recursive")
                        .short("r")
                        .long("depth")
                        .takes_value(true)
                        .help("Do recursive export, up to <depth>"),
                )
                .arg(
                    Arg::with_name("thin")
                        .short("t")
                        .long("thin")
                        .help("Thin export, only metadata necessary for reconstruction"),
                )
                .arg(
                    Arg::with_name("private")
                        .long("private")
                        .help("Include private objects"),
                ),
        )
        .subcommand(
            SubCommand::with_name("receive")
                .about("Imports objects")
                //TODO:  glob type
                .arg(
                    Arg::with_name("recursive")
                        .short("r")
                        .long("depth")
                        .takes_value(true)
                        .help("Constrain recursive import to <depth>"),
                )
                .arg(
                    Arg::with_name("thin")
                        .short("t")
                        .long("thin")
                        .help("Thin import, only metadata necessary for reconstruction"),
                )
                .arg(
                    Arg::with_name("private")
                        .long("private")
                        .help("Include private objects"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get-id")
                .about("Get the identifier on a object")
                .arg(
                    Arg::with_name("PATH")
                        .required(true)
                        .help("Path to a file in the objectstore"),
                ),
        )
        .subcommand(
            SubCommand::with_name("check")
                .about("Consistency check")
                .arg(
                    Arg::with_name("repair")
                        .short("r")
                        .long("repair")
                        .help("Try to repair damaged objects"),
                )
                .arg(
                    Arg::with_name("checksum")
                        .short("s")
                        .long("checksum")
                        .help("Check checksums"),
                ),
        )
}
