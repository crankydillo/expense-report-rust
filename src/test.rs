#![feature(plugin)]
extern crate postgres;

use postgres::{Connection, TlsMode};

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket;
extern crate r2d2_diesel;
extern crate r2d2_postgres;
extern crate r2d2;
extern crate chrono;
extern crate itertools;
use r2d2_postgres::PostgresConnectionManager;

mod db;
mod rest;
use rest::transaction;

use std::io::{self, BufRead};

fn main() -> () {
    let db_user = "";
    let db_pass = "";

    let conn_str = String::from("postgres://") + &db_user + &String::from(":") +
        &db_pass + &String::from("@localhost/gnucash");

    let manager = PostgresConnectionManager::new(conn_str.clone(), r2d2_postgres::TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(manager).expect("db pool");

    (0..100).for_each(|i| {
        let conn = db::pg_conn::PgConn(pool.get().unwrap());
        let tots = transaction::list(
            &conn,
            None,
            None,
            None
        );

        tots.acctSums.iter().for_each(|t| println!("{:?}", t));
    });

    let stdin = io::stdin();
    let line1 = stdin.lock().lines().next().unwrap().unwrap();
    print!("{}",line1);
}
