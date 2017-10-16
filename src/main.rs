extern crate postgres;
extern crate chrono;

mod account_dao;
mod transaction_dao;

//use AccountDao::AccountDao;
use postgres::{Connection, TlsMode};
use transaction_dao::TransactionDao;

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

    let conn = Connection::connect(conn_str.to_string(), TlsMode::None).unwrap();

    let dao = TransactionDao { i: 1 };
    let recs = dao.list(&conn, "", "", "");
    recs.iter().take(10).for_each(|r| println!("{:?}", r));
    println!("{}", recs.len());

    conn.finish();
}
