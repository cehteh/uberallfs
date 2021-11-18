use uberall::clap::{App, AppSettings, Arg, SubCommand};

pub fn optargs() -> App<'static, 'static> {
    SubCommand::with_name("objectstore")
        .about("Objectstore management")
        .arg(
            Arg::with_name("DIRECTORY")
                .required(true)
                .help("The objectstore directory"),
        )
        .setting(AppSettings::SubcommandRequired)
        .subcommand(init_optargs())
        .subcommand(lock_optargs())
        .subcommand(show_optargs())
        .subcommand(mkdir_optargs())
        .subcommand(gc_optargs())
        .subcommand(send_optargs())
        .subcommand(receive_optargs())
        .subcommand(getid_optargs())
        .subcommand(check_optargs())
}

fn init_optargs() -> App<'static, 'static> {
    SubCommand::with_name("init")
        .about("Initialize a new objectstore")
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .help("Force overwriting an existing directory"),
        )
        .arg(
            Arg::with_name("noroot")
                .long("no-root")
                .help("Initalize without an root directory"),
        )
        .arg(
            Arg::with_name("ARCHIVE")
                .short("i")
                .long("import")
                .takes_value(true)
                .help("Import root directory"),
        )
}

// Hidden option only for testing/diagnostics
fn lock_optargs() -> App<'static, 'static> {
    SubCommand::with_name("lock")
        .setting(AppSettings::Hidden)
        .about("Locks the objectstore (until CTRL-C is hit)")
        .arg(
            Arg::with_name("wait")
                .short("w")
                .long("wait")
                .help("Wait for the lock"),
        )
}

fn gc_optargs() -> App<'static, 'static> {
    SubCommand::with_name("gc")
        .about("Run the Garbage collector")
        .arg(
            Arg::with_name("dry-run")
                .long("dry-run")
                .short("n")
                .help("Don't remove any data, only show what would been done"),
        )
}

fn mkdir_optargs() -> App<'static, 'static> {
    SubCommand::with_name("mkdir")
        .about("Create a new directory")
        .arg(
            Arg::with_name("root")
                .long("force-root")
                .help("Creates a new root directory (dangerous!)"),
        )
        .arg(
            Arg::with_name("unlinked")
                .long("force")
                .short("f")
                .help("Creates a new directory object PATH is given"),
        )
        .arg(
            Arg::with_name("parents")
                .long("parents")
                .short("p")
                .help("make parent directories as needed"),
        )
        .arg(
            Arg::with_name("acl")
                .long("acl")
                .multiple(true)
                .takes_value(true) // optional? from parent? 'default'
                .help("Create a public shared directory"),
        )
        .arg(
            Arg::with_name("SOURCE")
                .long("link")
                .short("l")
                .takes_value(true)
                .help("Links to an existing directory"),
        )
        .arg(
            Arg::with_name("PATH")
                .takes_value(true)
                .help("The directory to create"),
        )
}

fn show_optargs() -> App<'static, 'static> {
    SubCommand::with_name("show")
        .about("Shows metadata about objects")
        .arg(
            Arg::with_name("PATH")
                .takes_value(true)
                .help("Path to investigate"),
        )
}

fn send_optargs() -> App<'static, 'static> {
    SubCommand::with_name("send")
        .about("Exports an object")
        .arg(
            Arg::with_name("ID_OR_PATH")
                .required(true)
                .help("The object to export"),
        )
        // TODO:  glob type compress
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
        )
}

fn receive_optargs() -> App<'static, 'static> {
    SubCommand::with_name("receive")
        .about("Imports objects")
        // TODO:  glob type
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
        )
}

fn getid_optargs() -> App<'static, 'static> {
    SubCommand::with_name("get-id")
        .about("Get the identifier on a object")
        .arg(
            Arg::with_name("PATH")
                .required(true)
                .help("Path to a file in the objectstore"),
        )
}

fn check_optargs() -> App<'static, 'static> {
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
        )
}
