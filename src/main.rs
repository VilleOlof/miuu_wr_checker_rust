#![warn(missing_docs)]

//! A program that fetches data regularly from the "Marble It Up! Ultra" backend.
//! to then send webhooks depending on new world records, new weekly challenges and weekly WR recaps.
//!
//! Read the README.md for the project and how to setup the config.
//!
//! Project hasn't been tested from the ground up with an empty database and config
//! So issues may appear there for now.

use anyhow::Result;
use miuu_wr_checker_rust::start;

/// To see actual main, go to ./lib.rs
fn main() -> Result<()> {
    start()
}
