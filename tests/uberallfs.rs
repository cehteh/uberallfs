use std::collections::BTreeMap;
use std::env::set_current_dir;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;
use uberall::{lazy_static::lazy_static, libc};

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
        .call_args(["-dd", "--version"])
        .assert_success()
        .assert_stdout_utf8("uberallfs 0.0.0");
}

#[test]
fn plumbing_init() {
    let mut uberallfs = TestCall::new(&EXECUTABLES, "uberallfs");
    let tempdir = TempDir::new().expect("created tempdir");
    uberallfs.current_dir(&tempdir);
    uberallfs
        .call_args(["-dd", "objectstore", "teststore/", "init"])
        .assert_success();
    uberallfs
        .call_args(["-dd", "objectstore", "teststore/", "init"])
        .assert_failure();
    uberallfs
        .call_args(["-dd", "objectstore", "teststore/", "init", "--force"])
        .assert_success();
}

#[test]
fn plumbing_basic() {
    let mut uberallfs = TestCall::new(&EXECUTABLES, "uberallfs");
    let tempdir = TempDir::new().expect("created tempdir");
    uberallfs.current_dir(&tempdir);
    uberallfs
        .call_args(["-dd", "objectstore", "teststore/", "init"])
        .assert_success();
    uberallfs
        .call_args(["-dd", "objectstore", "teststore/", "mkdir", "/"])
        .assert_exitcode(libc::EEXIST);
    uberallfs
        .call_args(["-dd", "objectstore", "teststore/", "mkdir", "/testdir"])
        .assert_success();
    uberallfs
        .call_args(["-dd", "objectstore", "teststore/", "mkdir", "/testdir"])
        .assert_exitcode(libc::EEXIST);
    uberallfs
        .call_args(["-dd", "objectstore", "teststore/", "show", "/testdir"])
        .assert_success();
    //PLANNED: -p is not implemented yet
    //uberallfs.call(["-dd", "objectstore", "teststore/", "mkdir", "-p", "/test/dir"]);
    uberallfs
        .call_args([
            "-dd",
            "objectstore",
            "teststore/",
            "show",
            "/doesnotexist",
        ])
        .assert_failure();
    uberallfs
        .call_args(["-dd", "objectstore", "teststore/", "show", "hasnoslash"])
        .assert_failure();
}
