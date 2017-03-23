extern crate upaste_server;
#[macro_use] extern crate clap;

use upaste_server::service;
use upaste_server::errors::*;

use clap::{Arg, App, SubCommand, ArgMatches};


pub fn main() {
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
                    .arg(Arg::with_name("silent")
                         .long("silent")
                         .help("Don't output any logging info")))
        .subcommand(SubCommand::with_name("cli")
                    .about("CLI functions")
                    .arg(Arg::with_name("clean")
                         .long("clean-stale")
                         .help("Clean out stale pastes")))
        .get_matches();

    if let Err(ref e) = run(matches) {
        use ::std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let stderr_msg = "Error writing to stderr";
        writeln!(stderr, "error: {}", e).expect(stderr_msg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(stderr_msg);
        }

        // `RUST_BACKTRACE=1`
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(stderr_msg);
        }

        ::std::process::exit(1);
    }
}


fn run(matches: ArgMatches) -> Result<()> {
    if let Some(serve_matches) = matches.subcommand_matches("serve") {
        let port = serve_matches.value_of("port").unwrap_or("3000");
        let host = format!("localhost:{}", port);
        let do_log = !matches.is_present("silent");
        service::start(&host, do_log);
        return Ok(());
    }

    if let Some(cli_matches) = matches.subcommand_matches("cli") {
        if cli_matches.is_present("clean") {
            println!("Cleaning!");
            unimplemented!();
        }
    }

    Ok(())
}
