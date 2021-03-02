use clap::{App, Arg, SubCommand};

pub fn args() -> App<'static, 'static> {
    SubCommand::with_name("objectstore")
        .about("Objectstore management")
        .arg(
            Arg::with_name("DIRECTORY")
                .required(true)
                .help("The objectstore directory"),
        )
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
            SubCommand::with_name("export")
                .about("Exports an object")
                .arg(
                    Arg::with_name("ID_OR_PATH")
                        .required(true)
                        .help("The object to export"),
                )
                .arg(
                    Arg::with_name("FILE_OR_DIR")
                        .required(true)
                        .help("Destination for export"),
                ), //TODO: recursive with depth && glob
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Imports an object")
                .arg(
                    Arg::with_name("FILE")
                        .required(true)
                        .help("Object archive to import"),
                ), //TODO: recursive with depth && glob
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
