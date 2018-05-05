#![feature(plugin)]
#![plugin(rocket_codegen)]

use pg_conn::PgConn;
use chrono::NaiveDate;
use db::transaction_dao::{Transaction, TransactionDao};
use serde_json;

#[get("/trans")]
fn list(conn: PgConn) -> String {
    let dao = TransactionDao { conn: &conn };
    let since = NaiveDate::from_ymd(2016, 10, 1).and_hms(0, 0, 0);
    let until = NaiveDate::from_ymd(2017, 10, 1).and_hms(0, 0, 0);
    let recs = dao.list(&since, &until, "");
    let v = recs.into_iter().take(10).collect::<Vec<Transaction>>();
    serde_json::to_string_pretty(&v).unwrap()
}


