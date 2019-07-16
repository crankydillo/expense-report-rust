/*#![feature(plugin)]
#![plugin(rocket_codegen)]

use chrono::NaiveDate;
use db::account_dao::AccountDao;
use pg_conn::PgConn;
use rocket_contrib::json::Json;
use serde_json;

#[get("/accounts")]
pub fn list(conn: PgConn) -> Json<Vec<String>> {
    let dao = AccountDao { conn: &conn };
    let since = NaiveDate::from_ymd(2016, 10, 1).and_hms(0, 0, 0);
    let until = NaiveDate::from_ymd(2017, 10, 1).and_hms(0, 0, 0);
    let recs = dao.list();
    let accountNames = recs.into_iter().map( |a| a.name ).collect::<Vec<String>>();
    Json(accountNames)
}
*/
