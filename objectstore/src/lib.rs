use clap::{ArgMatches};
use std::io;

mod optargs;
pub use self::optargs::optargs;

mod init;

extern crate log;

#[allow(unused_imports)]
use log::{debug, error, trace, info};

pub const VERSION: u16 = 0;

pub fn cmd(matches: &ArgMatches) -> io::Result<()> {
    let dir = matches.value_of_os("DIRECTORY").unwrap();

    trace!("dir: {:?}", dir);

    match matches.subcommand() {
        ("init", Some(sub_m)) => init::init(dir, sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    }
}




struct ObjectStore {
}

impl ObjectStore {
    //create
    //open

}

impl Drop for ObjectStore {
    fn drop(&mut self){
    }
}
