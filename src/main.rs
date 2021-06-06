#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate upaste_server;
#[macro_use]
extern crate clap;

use std::env;
use upaste_server::admin;
use upaste_server::service;

use clap::{App, Arg, SubCommand};

error_chain! {
    foreign_links {
        Upaste(upaste_server::errors::Error);
    }
    errors {}
}

pub fn run() -> Result<()> {
    let matches = App::new("upaste")
        .version(crate_version!())
        .about("uPaste Server")
        .subcommand(SubCommand::with_name("serve")
                    .about("Initialize Server"))
        .subcommand(SubCommand::with_name("admin")
                    .about("Admin functions")
                    .subcommand(SubCommand::with_name("database")
                        .about("Database functions")
                        .subcommand(SubCommand::with_name("migrate")
                            .about("Look for and apply any available un-applied migrations"))
                        .subcommand(SubCommand::with_name("shell")
                            .about("Open a database shell")))
                    .subcommand(SubCommand::with_name("clean-before")
                        .about("Clean out stale pastes by date or number of days")
                        .arg(Arg::with_name("database")
                             .long("db-path")
                             .takes_value(true)
                             .help("Sqlite database path to connect to"))
                        .arg(Arg::with_name("date")
                             .long("--date")
                             .takes_value(true)
                             .help("Clean out stale pastes before a given date (yyyy-mm-dd)"))
                        .arg(Arg::with_name("days")
                             .long("--days")
                             .takes_value(true)
                             .help("Clean out stale pastes before a number of days prior to now ([0-9])"))
                        .arg(Arg::with_name("no-confirm")
                             .long("no-confirm")
                             .takes_value(false)
                             .help("Auto-confirm/skip any confirmation checks"))))
        .get_matches();

    if matches.subcommand_matches("serve").is_some() {
        service::start()?;
        return Ok(());
    }

    if let Some(admin_matches) = matches.subcommand_matches("admin") {
        admin::handle(admin_matches)?;
        return Ok(());
    }

    println!("upaste: see `--help`");
    Ok(())
}

pub fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}
