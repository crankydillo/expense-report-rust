#![feature(plugin)]
#![plugin(rocket_codegen)]

use pg_conn::PgConn;
use chrono::NaiveDate;
use db::account_dao::AccountDao;
use serde_json;

#[get("/accounts", format="application/json")]
pub fn list(conn: PgConn) -> String {
    let dao = AccountDao { conn: &conn };
    let since = NaiveDate::from_ymd(2016, 10, 1).and_hms(0, 0, 0);
    let until = NaiveDate::from_ymd(2017, 10, 1).and_hms(0, 0, 0);
    let recs = dao.list();
    let accountNames = recs.into_iter().map( |a| a.name ).collect::<Vec<String>>();
    serde_json::to_string(&accountNames).unwrap()
}

