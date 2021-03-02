#[macro_use]
extern crate clap;
use clap::{Arg, App};

mod args;
pub use self::args::uberallfs_args;


use objectstore;




fn main() {
    let matches = uberallfs_args()
        .subcommand(objectstore::args())
        .get_matches();
}
