#[allow(dead_code)]
mod common;

use assert_cmd::prelude::*;
use common::oav_command;
use predicates::prelude::*;

#[test]
fn bare_oav_without_tty_prints_help() {
    oav_command()
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: oav"));
}
