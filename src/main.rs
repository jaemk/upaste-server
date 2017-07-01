extern crate upaste_server;
#[macro_use] extern crate clap;

use std::env;
use upaste_server::service;
use upaste_server::admin;
use upaste_server::errors::*;

use clap::{Arg, App, SubCommand, ArgMatches};


pub fn main() {
    let matches = App::new("upaste")
        .version(crate_version!())
        .about("uPaste Server")
        .subcommand(SubCommand::with_name("serve")
                    .about("Initialize Server")
                    .arg(Arg::with_name("database")
                        .long("db-url")
                        .takes_value(true)
                        .help("Postgres database URL to connect to"))
                    .arg(Arg::with_name("port")
                        .long("port")
                        .short("p")
                        .takes_value(true)
                        .help("Port to listen on. Defaults to 3000"))
                    .arg(Arg::with_name("public")
                        .long("public")
                        .help("Serve on '0.0.0.0' instead of 'localhost'"))
                    .arg(Arg::with_name("log")
                        .long("log")
                        .help("Output logging info. Shortcut for setting env-var LOG=info")))
        .subcommand(SubCommand::with_name("admin")
                    .about("admin functions")
                    .subcommand(SubCommand::with_name("migrate")
                        .about("Look for and apply any available un-applied migrations"))
                    .subcommand(SubCommand::with_name("shell")
                        .about("Open a database shell"))
                    .subcommand(SubCommand::with_name("clean-before")
                        .about("Clean out stale pastes by date or number of days")
                        .arg(Arg::with_name("database")
                             .long("db-url")
                             .takes_value(true)
                             .help("Postgres database URL to connect to"))
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

    if let Err(ref e) = run(matches) {
        use ::std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let stderr_msg = "Error writing to stderr";
        writeln!(stderr, "[ERROR] {}", e).expect(stderr_msg);

        ::std::process::exit(1);
    }
}


fn run(matches: ArgMatches) -> Result<()> {
    if let Some(serve_matches) = matches.subcommand_matches("serve") {
        if serve_matches.is_present("log") {
            env::set_var("LOG", "info");
        }

        let port = serve_matches.value_of("port").unwrap_or("3000");
        let host_base = if serve_matches.is_present("public") { "0.0.0.0" } else { "localhost" };
        let host = format!("{}:{}", host_base, port);
        let db_url = serve_matches.value_of("database");
        service::start(&host, db_url);
        return Ok(());
    }

    if let Some(admin_matches) = matches.subcommand_matches("admin") {
        return admin::handle(admin_matches)
    }

    Ok(())
}
