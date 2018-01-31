#![recursion_limit = "1024"]
#[macro_use] extern crate error_chain;
extern crate upaste_server;
#[macro_use] extern crate clap;

use std::env;
use upaste_server::service;
use upaste_server::admin;

use clap::{Arg, App, SubCommand};


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
                    .about("Initialize Server")
                    .arg(Arg::with_name("port")
                        .long("port")
                        .short("p")
                        .takes_value(true)
                        .help("Port to listen on. Defaults to 3000"))
                    .arg(Arg::with_name("public")
                        .long("public")
                        .help("Serve on '0.0.0.0' instead of 'localhost'"))
                    .arg(Arg::with_name("debug")
                        .long("debug")
                        .help("Output debug log info. Shortcut for setting env-var LOG=debug")))
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

    if let Some(serve_matches) = matches.subcommand_matches("serve") {
        env::set_var("LOG", "info");
        if serve_matches.is_present("debug") {
            env::set_var("LOG", "debug");
        }

        let port = serve_matches.value_of("port").unwrap_or("3000");
        let host_base = if serve_matches.is_present("public") { "0.0.0.0" } else { "localhost" };
        let host = format!("{}:{}", host_base, port);
        service::start(&host)?;
        return Ok(());
    }

    if let Some(admin_matches) = matches.subcommand_matches("admin") {
        admin::handle(admin_matches)?;;
        return Ok(());
    }

    println!("upaste: see `--help`");
    Ok(())
}


quick_main!(run);

