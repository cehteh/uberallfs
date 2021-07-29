#[macro_use]
extern crate lazy_static;
use std::collections::BTreeMap;
use std::env::set_current_dir;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

use bintest::BinTest;

use testcall::*;

lazy_static! {
    static ref EXECUTABLES: BinTest = BinTest::new();
}

#[test]
fn test_version() {
    let uberallfs = TestCall::new(&EXECUTABLES, "uberallfs");

    // check for version as remider to keep the tests up to date
    uberallfs
        .call(["-d", "-v", "--version"], NO_ENVS)
        .assert_success()
        .assert_stdout_utf8("uberallfs 0.0.0");
}

#[test]
fn plumbing_init() {
    let mut uberallfs = TestCall::new(&EXECUTABLES, "uberallfs");
    let tempdir = TempDir::new().expect("created tempdir");
    uberallfs.current_dir(&tempdir);
    uberallfs
        .call(["-d", "-v", "objectstore", "ubatest", "init"], NO_ENVS)
        .assert_success();
    uberallfs
        .call(["-d", "-v", "objectstore", "ubatest", "init"], NO_ENVS)
        .assert_failure();
    uberallfs
        .call(
            ["-d", "-v", "objectstore", "ubatest", "init", "--force"],
            NO_ENVS,
        )
        .assert_success();
}

#[test]
fn plumbing_basic() {
    let mut uberallfs = TestCall::new(&EXECUTABLES, "uberallfs");
    let tempdir = TempDir::new().expect("created tempdir");
    uberallfs.current_dir(&tempdir);
    uberallfs
        .call(["-d", "-v", "objectstore", "ubatest", "init"], NO_ENVS)
        .assert_success();
    uberallfs
        .call(
            ["-d", "-v", "objectstore", "ubatest", "mkdir", "/testdir"],
            NO_ENVS,
        )
        .assert_success();
    uberallfs
        .call(
            ["-d", "-v", "objectstore", "ubatest", "show", "/testdir"],
            NO_ENVS,
        )
        .assert_success();
    //PLANNED: -p is not implemented yet
    //uberallfs.call(["-d", "-v", "objectstore", "ubatest", "mkdir", "-p", "/test/dir"]);
    //FIXME: uberallfs.fail(["-d", "-v", "objectstore", "ubatest", "show", "/doesnotexist"]);
    uberallfs
        .call(
            ["-d", "-v", "objectstore", "ubatest", "show", "hasnoslash"],
            NO_ENVS,
        )
        .assert_failure();
}
