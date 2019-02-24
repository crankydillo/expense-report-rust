#![feature(plugin)]
//#![plugin(rocket_codegen)]
#![allow(warnings)]
#![feature(proc_macro_hygiene, decl_macro)]

extern crate chrono;
extern crate itertools;
extern crate serde;
extern crate serde_json;
extern crate postgres;
#[macro_use] extern crate diesel;
extern crate r2d2_diesel;
extern crate r2d2_postgres;
extern crate r2d2;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;

mod db;
use db::{
    account_dao,
    db_conn,
    pg_conn,
    transaction_dao
};

mod rest;
use rest::{
    account,
    transaction
};

use postgres::{Connection, TlsMode};
use pg_conn::PgConn;
use r2d2_postgres::PostgresConnectionManager;

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

    let manager = PostgresConnectionManager::new(conn_str.clone(), r2d2_postgres::TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(manager).expect("db pool");

    rocket::ignite()
        .mount("/", routes![
               account::list,
               transaction::list,
               transaction::monthly_totals,
        ]).manage(conn_str)
        .manage(pool)
        .launch();
}
