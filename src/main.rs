#![feature(plugin)]
#![plugin(rocket_codegen)]
#![allow(warnings)]

extern crate chrono;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate postgres;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate r2d2_diesel;
extern crate r2d2_postgres;
extern crate r2d2;
#[macro_use] extern crate serde_derive;

mod account_dao;
mod transaction_dao;
mod db_conn;
mod pg_conn;

use chrono::{NaiveDate, NaiveDateTime};
use rocket::State;
use rocket_contrib::Json;
use postgres::{Connection, TlsMode};
use transaction_dao::{Transaction, TransactionDao};
use pg_conn::PgConn;
use r2d2_postgres::PostgresConnectionManager;

use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

#[get("/bye")]
fn bye(conn: PgConn) -> String {
    let dao = TransactionDao {};
    let since = NaiveDate::from_ymd(2016, 10, 1).and_hms(0, 0, 0);
    let until = NaiveDate::from_ymd(2017, 10, 1).and_hms(0, 0, 0);
    let recs = dao.list(&conn, &since, &until, "");
    let v = recs.into_iter()
        .collect::<Vec<Transaction>>();
    println!("{}", v.len());
    serde_json::to_string(&v).unwrap()
}

fn main() -> () {

    let args: Vec<_> = std::env::args().collect();
    if (args.len() != 3) {
        println!("Usage: expense-report <db_user> <db_pass>");
        std::process::exit(1);
    };

    let db_user = &args[1];
    let db_pass = &args[2];

    let conn_str = String::from("postgres://") + &db_user + &String::from(":") +
        &db_pass + &String::from("@localhost/gnucash");

    let config = r2d2::Config::default();
    let manager = PostgresConnectionManager::new(conn_str.clone(), r2d2_postgres::TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(config, manager).expect("db pool");

    #[get("/hi")]
    fn hi(conn_str: State<String>) -> String {
        let conn = Connection::connect(conn_str.to_string(), TlsMode::None).unwrap();
        let dao = TransactionDao {};
        let since = NaiveDate::from_ymd(2016, 10, 1).and_hms(0, 0, 0);
        let until = NaiveDate::from_ymd(2017, 10, 1).and_hms(0, 0, 0);
        let recs = dao.list(&conn, &since, &until, "");
        let v = recs.into_iter().take(10).collect::<Vec<Transaction>>();
        println!("{:?}", conn.finish());
        serde_json::to_string_pretty(&v).unwrap()
    }

    rocket::ignite()
        .mount("/", routes![hi, bye])
        .manage(conn_str)
        .manage(pool)
        .launch();
}
