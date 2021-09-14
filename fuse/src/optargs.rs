use uberall::clap::{App, AppSettings, Arg, SubCommand};

pub fn optargs() -> App<'static, 'static> {
    SubCommand::with_name("fuse")
        .about("fuse frontend")
        .arg(
            Arg::with_name("MOUNTPOINT")
                .required(true)
                .help("The mountpoint"),
        )
        .setting(AppSettings::SubcommandRequired)
        .subcommand(mount_optargs())
        .subcommand(umount_optargs())
}

fn mount_optargs() -> App<'static, 'static> {
    SubCommand::with_name("mount")
        .about("Mount the filesystem")
        .arg(Arg::with_name("OBJECTSTORE").help("The objectstore directory"))
        .arg(
            Arg::with_name("offline")
                .short("n")
                .long("offline")
                .help("Start without the network node"),
        )
        .arg(
            Arg::with_name("root")
                .short("r")
                .long("root")
                .takes_value(true)
                .help("Path to an alternative root directory"),
        )
}

fn umount_optargs() -> App<'static, 'static> {
    SubCommand::with_name("umount")
        .about("Unmount the filesystem")
        .arg(
            Arg::with_name("lazy")
                .short("l")
                .long("lazy")
                .help("Do lazy unmounting"),
        )
}
