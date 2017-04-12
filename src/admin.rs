/*!
General Admin commands

*/
use std::io::Write;

use errors::*;
use service;

use clap::ArgMatches;
use chrono::{UTC, NaiveDate, TimeZone, DateTime};
use time::Duration;

use diesel;
use diesel::prelude::*;


/// Print a message and require y/n confirmation
fn confirm(msg: &str) -> Result<()> {
    print!("{}", msg);
    ::std::io::stdout().flush().expect("Error flushing stdout");
    let mut input = String::new();
    let stdin = ::std::io::stdin();
    stdin.read_line(&mut input).expect("Error reading stdin");
    if input.trim().to_lowercase() == "y" { return Ok(()) }
    bail!("error confirming");
}


/// Delete stale pastes that haven't been viewed prior to a given date.
fn delete_pastes_before(date: DateTime<UTC>, no_confirm: bool) -> Result<()> {
    let conn = service::establish_connection(None);
    use schema::pastes::dsl::*;

    //let count = pastes.select(key)
    //                  .filter(date_viewed.lt(date))
    //                  .load::<String>(&conn)
    //                  .chain_err(|| "Error counting pastes...")?
    //                  .len();
    let count = pastes.filter(date_viewed.lt(date))
                      .count()
                      .get_result::<i64>(&conn)
                      .chain_err(|| "Error counting pastes...")?;
    println!("** Found {} pastes that weren't viewed since {} **", count, date);

    if !no_confirm {
        let conf = confirm(&format!("Are you sure you want to delete {} pastes that weren't viewed since {}? (y/n) >> ", count, date));
        if !conf.is_ok() { return Ok(()) }
    }

    let n_deleted = diesel::delete(pastes.filter(date_viewed.lt(date)))
        .execute(&conn)
        .chain_err(|| "Error deleting posts...")?;
    println!("** {} pastes deleted", n_deleted);
    Ok(())
}


pub fn handle(matches: &ArgMatches) -> Result<()> {
    let no_confirm = matches.is_present("no-confirm");
    if let Some(v) = matches.value_of("clean-before-date") {
        let date = {
            let date = NaiveDate::parse_from_str(v, "%Y-%m-%d")
                .chain_err(|| format!("Invalid timestamp format (yyyy-mm-dd): {}", v))?;
            let date = UTC.from_utc_date(&date);
            date.and_hms(0, 0, 0)
        };
        delete_pastes_before(date, no_confirm).chain_err(|| "Error deleting pastes")?;
        return Ok(())
    }

    if let Some(v) = matches.value_of("clean-before-days") {
        let n = v.parse::<u32>().chain_err(|| "Invalid integer")?;
        let date = UTC::now() - Duration::seconds(60*60*24*n as i64);
        delete_pastes_before(date, no_confirm).chain_err(|| "Error deleting pastes")?;
        return Ok(())
    }

    Ok(())
}
