/*!
General Admin commands

*/
use std::io::Write;

use errors::*;
use service;

use clap::ArgMatches;
use chrono::{UTC, NaiveDate, TimeZone};
use time::Duration;

use diesel;
use diesel::prelude::*;


fn confirm(msg: &str) -> Result<()> {
    print!("{}", msg);
    ::std::io::stdout().flush().expect("Error flushing stdout");
    let mut input = String::new();
    let stdin = ::std::io::stdin();
    stdin.read_line(&mut input).expect("Error reading stdin");
    if input.trim().to_lowercase() == "y" { return Ok(()) }
    bail!("error confirming");
}


pub fn handle(matches: &ArgMatches) -> Result<()> {

    if matches.is_present("clean") {
        let date = match matches.value_of("clean") {
            Some(v) if v == "now" => UTC::now(),
            Some(v) => {
                let date = NaiveDate::parse_from_str(v, "%Y-%m-%d")
                    .chain_err(|| format!("Invalid timestamp format (yyyy-mm-dd): {}", v))?;
                let date = UTC.from_utc_date(&date);
                date.and_hms(0, 0, 0)
            }
            None => UTC::now() - Duration::seconds(60*60*24*30),
        };
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

        let conf = confirm(&format!("Are you sure you want to delete {} pastes that weren't viewed since {}? (y/n) >> ", count, date));
        if !conf.is_ok() { return Ok(()) }

        let n_deleted = diesel::delete(pastes.filter(date_viewed.lt(date)))
            .execute(&conn)
            .chain_err(|| "Error deleting posts...")?;
        println!("** {} pastes deleted", n_deleted);

        return Ok(())
    }
    Ok(())
}
