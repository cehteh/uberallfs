mod args;
pub use self::args::args;

use std::ffi::OsStr;

use clap::{ArgMatches};

#[macro_use]
extern crate log;
use log::{debug, error, trace, info};

pub fn cmd(matches: &ArgMatches) {

    let dir = matches.value_of_os("DIRECTORY").unwrap();

    trace!("dir: {:?}", dir);

    match matches.subcommand() {
        ("init", Some(sub_m)) => cmd_init(dir, sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    }
}


pub fn cmd_init(dir: &OsStr, matches: &ArgMatches) {

    // if exists / force /mkdir
    //mkdir
    // if failed then if exists and force
    //   and it is a objectdir

    // initialize

    // unpack and verify import
    
    // or create a new root  --local or --public
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
