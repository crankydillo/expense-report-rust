#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate postgres;

#[macro_use]
extern crate serde_derive;

mod account_dao;
mod transaction_dao;

use rocket::State;
use rocket_contrib::Json;
use postgres::{Connection, TlsMode};
use transaction_dao::{Transaction, TransactionDao};

#[get("/bye")]
fn bye() -> &'static str {
    "bye\n"
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

    #[get("/hi")]
    fn hi(conn_str: State<String>) -> String {
        let conn = Connection::connect(conn_str.to_string(), TlsMode::None).unwrap();
        let dao = TransactionDao { i: 1 };
        let recs = dao.list(&conn, "", "", "");
        let v = recs.into_iter().take(10).collect::<Vec<Transaction>>();
        println!("{:?}", conn.finish());
        serde_json::to_string_pretty(&v).unwrap()
    }

    rocket::ignite()
        .mount("/", routes![hi, bye])
        .manage(conn_str)
        .launch();
}
