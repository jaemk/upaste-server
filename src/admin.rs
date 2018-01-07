/*!
General Admin commands

*/
use std::env;
use std::path;
use std::io::Write;

use migrant_lib::{self, Config, DbKind};
use clap::ArgMatches;
use chrono::{Utc, NaiveDate, TimeZone, DateTime};
use time::Duration;

use errors::*;
use models;
use service;

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
fn delete_pastes_before<T: AsRef<path::Path>>(date: DateTime<Utc>, no_confirm: bool, database_path: T) -> Result<()> {
    let conn = service::establish_connection(database_path.as_ref());

    let count = models::Paste::count_outdated(&conn, &date)?;
    println!("** Found {} pastes that weren't viewed since {} **", count, date);

    if !no_confirm {
        let conf = confirm(&format!("Are you sure you want to delete {} pastes that weren't viewed since {}? [y/n] ", count, date));
        if conf.is_err() { return Ok(()) }
    }

    let n_deleted = models::Paste::delete_outdated(&conn, &date)?;
    println!("** {} pastes deleted", n_deleted);
    Ok(())
}


pub fn handle(matches: &ArgMatches) -> Result<()> {
    if let Some(db_matches) = matches.subcommand_matches("database") {
        let dir = env::current_dir()?;
        let config_path = match migrant_lib::search_for_settings_file(&dir) {
            None => {
                Config::init_in(&dir)
                    .database_type(DbKind::Sqlite)
                    .initialize()?;
                match migrant_lib::search_for_settings_file(&dir) {
                    None => bail!("Unable to find `Migrant.toml` even though it was just saved."),
                    Some(p) => p,
                }
            }
            Some(p) => p,
        };

        let mut config = Config::from_settings_file(&config_path)?;

        if db_matches.is_present("setup") {
            config.setup()?;
            return Ok(())
        }


        match db_matches.subcommand() {
            ("shell", _) => {
                migrant_lib::shell(&config)?;
            }
            ("migrate", _) => {
                // load applied migrations from the database
                let config = config.reload()?;

                let res = migrant_lib::Migrator::with_config(&config)
                    .direction(migrant_lib::Direction::Up)
                    .all(true)
                    .apply();
                if let Err(ref err) = res {
                    if err.is_migration_complete() {
                        println!("Database is up-to-date!");
                        return Ok(());
                    }
                }
                let _ = res?;
                return Ok(())
            }
            _ => println!("see `--help`"),
        }

        return Ok(())
    }

    if let Some(matches) = matches.subcommand_matches("clean-before") {
        let no_confirm = matches.is_present("no-confirm");
        let database_path = match matches.value_of("database") {
            Some(p) => path::PathBuf::from(p),
            None => service::migrant_database_path()
                .ok_or_else(|| format_err!(ErrorKind::Msg, "No config file found"))?,
        };
        if let Some(v) = matches.value_of("date") {
            let date = {
                let date = NaiveDate::parse_from_str(v, "%Y-%m-%d")
                    .chain_err(|| format!("Invalid timestamp format (yyyy-mm-dd): {}", v))?;
                let date = Utc.from_utc_date(&date);
                date.and_hms(0, 0, 0)
            };
            delete_pastes_before(date, no_confirm, &database_path)?;
            return Ok(())
        }

        if let Some(v) = matches.value_of("days") {
            let n = v.parse::<u32>()?;
            let date = Utc::now() - Duration::seconds(60*60*24*n as i64);
            delete_pastes_before(date, no_confirm, &database_path)?;
            return Ok(())
        }
    }

    println!("See: upaste admin --help");
    Ok(())
}
