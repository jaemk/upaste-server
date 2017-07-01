/*!
General Admin commands

*/
use std::env;
use migrant_lib::{self, Config};
use std::io::Write;

use errors::*;
use models;
use service;

use clap::ArgMatches;
use chrono::{UTC, NaiveDate, TimeZone, DateTime};
use time::Duration;


/// Print a message and require y/n confirmation
fn confirm(msg: &str) -> Result<()> {
    print!("{}", msg);
    ::std::io::stdout().flush().expect("Error flushing stdout");
    let mut input = String::new();
    let stdin = ::std::io::stdin();
    stdin.read_line(&mut input).expect("Error reading stdin");
    if input.trim().to_lowercase() == "y" { return Ok(()) }
    bail!("Unable to confirm");
}


/// Delete stale pastes that haven't been viewed prior to a given date.
fn delete_pastes_before(date: DateTime<UTC>, no_confirm: bool, database_url: Option<&str>) -> Result<()> {
    let conn = service::establish_connection(database_url);

    let count = models::Paste::count_outdated(&conn, &date)?;
    println!("** Found {} pastes that weren't viewed since {} **", count, date);

    if !no_confirm {
        let conf = confirm(&format!("Are you sure you want to delete {} pastes that weren't viewed since {}? (y/n) >> ", count, date));
        if !conf.is_ok() { return Ok(()) }
    }

    let n_deleted = models::Paste::delete_outdated(&conn, &date)?;
    println!("** {} pastes deleted", n_deleted);
    Ok(())
}


pub fn handle(matches: &ArgMatches) -> Result<()> {
    let no_confirm = matches.is_present("no-confirm");
    let database_url = matches.value_of("database");
    if matches.is_present("migrate") {
        let dir = env::current_dir()
            .map_err(|e| format_err!("failed to get current directory -> {}", e))?;
        match migrant_lib::search_for_config(&dir) {
            None => {
                Config::init(&dir)
                    .map_err(|e| format_err!("failed to initialize project -> {}", e))?;
            }
            Some(p) => {
                let config = Config::load(&p).expect("failed to load config");
                let  res = migrant_lib::Migrator::with_config(&config)
                    .direction(migrant_lib::Direction::Up)
                    .all(true)
                    .apply();
                if let Err(ref err) = res {
                    if let migrant_lib::Error::MigrationComplete(_) = *err {
                        println!("Database is up-to-date!");
                        return Ok(());
                    }
                }
                let _ = res?;
            }
        };
        return Ok(())
    }

    if let Some(v) = matches.value_of("clean-before-date") {
        let date = {
            let date = NaiveDate::parse_from_str(v, "%Y-%m-%d")
                .map_err(|e| format_err!("Invalid timestamp format (yyyy-mm-dd): {} -- {}", v, e))?;
            let date = UTC.from_utc_date(&date);
            date.and_hms(0, 0, 0)
        };
        delete_pastes_before(date, no_confirm, database_url)?;
        return Ok(())
    }

    if let Some(v) = matches.value_of("clean-before-days") {
        let n = v.parse::<u32>()?;
        let date = UTC::now() - Duration::seconds(60*60*24*n as i64);
        delete_pastes_before(date, no_confirm, database_url)?;
        return Ok(())
    }

    Ok(())
}
