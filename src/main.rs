#![allow(warnings)]
extern crate actix_web;

use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder};
use diesel::r2d2::{self, ConnectionManager};
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
use diesel::prelude::*;

mod db;
use crate::db::*;
use crate::db::transaction_dao::*;

#[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;
use models::*;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    let dbPath = "/home/samuel/Documents/gnucash/sqlite3/finances.gnucash";
    let manager = ConnectionManager::<SqliteConnection>::new(dbPath);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(index)
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}

#[get("/{id}/{name}")]
pub async fn index(
    pool: web::Data<DbPool>,
    info: web::Path<(String, String)>
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let tran_dao = TransactionDao { conn: &conn };
    let trans = tran_dao.list();

    Ok(HttpResponse::Ok().json(trans))
}

/*
mod routes {

    pub mod budget {
        use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder};
        use chrono::Local;
        use crate::db::budget_dao::BudgetDao;
        use rest::transaction;
        use std::collections::HashSet;
        use std::iter::FromIterator;

        #[get("/budget")]
        pub fn get(
            pool: web::Data<DbPool>,
        ) -> Result<HttpResponse, Error> {
            let conn = pool.get().expect("couldn't get db connection from pool");
            let budget_dao = BudgetDao { conn: &conn };
            let now = Local::now().naive_local().date();
            // TODO deal with option
            let budget = budget_dao.get_curr(now).unwrap();
            let monthly_totals =
                transaction::monthly_totals(
                    &conn,
                    None,
                    None,
                    Some("1".to_string()),
                    None
                );

            // For small collections, people claim this is faster than a map
            let get_monthly_total = |qualified_account_name: String| {
                monthly_totals.acctSums.iter().find(|mt| {
                    mt.name == qualified_account_name
                }).map(|mt| mt.total)
                .unwrap_or(0)
            };

            let budgeted_names: HashSet<String> = HashSet::from_iter(
                budget.amounts.iter().map(|ba| ba.account.qualified_name())
            );

            let mut budget_amounts: Vec<BudgetAmount> = budget.amounts.into_iter().map(|a| {
                BudgetAmount {
                    name: a.account.qualified_name(),
                    in_budget: true,
                    budgeted: a.amount,
                    actual: get_monthly_total(a.account.qualified_name()),
                }
            }).collect();

            // Add in the expenses that weren't budgeted
            monthly_totals.acctSums.into_iter()
                .filter(|mt| !budgeted_names.contains(&mt.name))
                .for_each(|mt| {
                    budget_amounts.push(
                        BudgetAmount {
                            name: mt.name,
                            in_budget: false,
                            budgeted: 0,
                            actual: mt.total
                        }
                    );
                });


            HttpResponse::Ok().json(
                Budget {
                    name: budget.name,
                    amounts: budget_amounts
                });

        }

        #[derive(Debug)]
        #[derive(Serialize)]
        pub struct Budget {
            pub name: String,
            pub amounts: Vec<BudgetAmount>
        }

        #[derive(Debug)]
        #[derive(Serialize)]
        pub struct BudgetAmount {
            pub name: String,
            pub in_budget: bool,
            pub budgeted: i64,
            pub actual: i64
        }
    }

    pub mod transaction {
        use db::pg_conn::PgConn;
        use db::transaction_dao::Transaction;
        use rest::transaction::TranSplit;
        use rest::transaction;
        use rocket_contrib::json::Json;

        #[get("/monthly-totals?<since>&<until>&<months>&<year>")]
        pub fn monthly_totals<'a>(
            pool: web::Data<DbPool>,
            since: Option<String>,
            until: Option<String>,
            months: Option<String>,
            year: Option<String>
        ) -> Result<HttpResponse, Error> {
            HttpResponse::Ok().json(transaction::monthly_totals(&conn, since, until, months, year))
        }

        #[get("/trans?<since>&<until>&<months>&<year>")]
        pub fn list(
            pool: web::Data<DbPool>,
            since: Option<String>,
            until: Option<String>,
            months: Option<String>,
            year: Option<String>
        ) -> Result<HttpResponse, Error> {
            HttpResponse::Ok().json(transaction::list(conn, since, until, months, year))
        }

        #[get("/expenses/<expense_name>/<month>")]
        pub fn expense_splits<'a>(
            pool: web::Data<DbPool>,
            expense_name: String,
            month: String
        ) -> Result<HttpResponse, Error> {
            let splits = transaction::expense_splits(conn, expense_name, month);
            let exp_splits = ExpenseSplits {
                msg: "hi!".to_string(),
                splits: splits
            };
            HttpResponse::Ok().json(exp_splits)
        }

        #[derive(Debug)]
        #[derive(Serialize)]
        pub struct ExpenseSplits {
            pub msg: String,
            pub splits: Vec<TranSplit>
        }
    }

    pub mod account {
        use chrono::NaiveDate;
        use db::account_dao::AccountDao;
        use pg_conn::PgConn;
        use rocket_contrib::json::Json;
        use serde_json;

        #[get("/accounts")]
        pub fn list(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
            let dao = AccountDao { conn: &conn };
            let since = NaiveDate::from_ymd(2016, 10, 1).and_hms(0, 0, 0);
            let until = NaiveDate::from_ymd(2017, 10, 1).and_hms(0, 0, 0);
            let recs = dao.list();
            let accountNames = recs.into_iter().map( |a| a.name ).collect::<Vec<String>>();
            HttpResponse::Ok().json(accountNames)
        }
    }
}
*/
