#![allow(warnings)]
extern crate actix_web;

use actix_files as fs;
use actix_web::{App, HttpServer};

mod db;
mod rest;

use crate::db::*;
use crate::db::transaction_dao::*;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    let args: Vec<_> = std::env::args().collect();
    if (args.len() != 2) {
        println!("Usage: expense-report <path_to_sqlite_db>");
        std::process::exit(1);
    };

    let db_path = &args[1];

    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(get)
            .service(routes::budget::get)
            .service(routes::transaction::monthly_totals)
            .service(routes::transaction::list)
            .service(routes::transaction::expense_splits)
            .service(routes::account::list)
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}

#[actix_web::get("/hi")]
pub async fn get() -> impl actix_web::Responder {
    "Hello"
}
 
mod routes {

    pub mod budget {
        use r2d2::Pool;
        use r2d2_sqlite::SqliteConnectionManager;
        use actix_web::{get, web, App, Error, HttpResponse, HttpServer};
        use actix_web::web::Query;
        use chrono::Local;
        use crate::db::budget_dao::BudgetDao;
        use crate::rest::transaction;
        use std::collections::HashSet;
        use std::iter::FromIterator;
        use ::serde::Serialize;

        #[get("/res/budget")]
        pub async fn get(
            pool: web::Data<Pool<SqliteConnectionManager>>,
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


            Ok(HttpResponse::Ok().json(
                Budget {
                    name: budget.name,
                    amounts: budget_amounts
                }))
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
        use r2d2::Pool;
        use r2d2_sqlite::SqliteConnectionManager;
        use actix_web::*;
        use actix_web::web::Query;
        use chrono::Local;
        use crate::db::budget_dao::BudgetDao;
        use crate::rest::transaction::TranSplit;
        use crate::rest::transaction;
        use std::iter::FromIterator;
        use ::serde::*;


        #[derive(Deserialize)]
        struct MtParams {
            since: Option<String>,
            until: Option<String>,
            months: Option<String>,
            year: Option<String>,
        }

        #[get("/res/monthly-totals")]
        pub async fn monthly_totals<'a>(
            pool: web::Data<Pool<SqliteConnectionManager>>,
            params: Query<MtParams>,
        ) -> Result<HttpResponse, Error> {
            let conn = pool.get().expect("couldn't get db connection from pool");
            Ok(HttpResponse::Ok().json(
                    transaction::monthly_totals(
                        &conn,
                        params.since.clone(),
                        params.until.clone(),
                        params.months.clone(),
                        params.year.clone()
                    )
                )
            )
        }

        #[get("/res/trans")]
        pub async fn list(
            pool: web::Data<Pool<SqliteConnectionManager>>,
            params: Query<MtParams>,
        ) -> Result<HttpResponse, Error> {
            let conn = pool.get().expect("couldn't get db connection from pool");
            Ok(HttpResponse::Ok().json(
                    transaction::list(
                        &conn,
                        params.since.clone(),
                        params.until.clone(),
                        params.months.clone(),
                        params.year.clone()
                    )
                )
            )
        }

        #[get("/res/expenses/{expense_name}/{month}")]
        pub async fn expense_splits<'a>(
            pool: web::Data<Pool<SqliteConnectionManager>>,
            path: web::Path<(String, String)>
        ) -> Result<HttpResponse, Error> {
            let expense_name = path.0.clone();
            let month = path.1.clone();
            let conn = pool.get().expect("couldn't get db connection from pool");
            let splits = transaction::expense_splits(&conn, expense_name, month);
            let exp_splits = ExpenseSplits {
                msg: "hi!".to_string(),
                splits: splits
            };
            Ok(HttpResponse::Ok().json(exp_splits))
        }

        #[derive(Debug)]
        #[derive(Serialize)]
        pub struct ExpenseSplits {
            pub msg: String,
            pub splits: Vec<TranSplit>
        }
    }

    pub mod account {
        use r2d2::Pool;
        use r2d2_sqlite::SqliteConnectionManager;
        use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder};
        use chrono::*;
        use crate::db::account_dao::AccountDao;
        use crate::rest::transaction;

        #[get("/res/accounts")]
        pub async fn list(pool: web::Data<Pool<SqliteConnectionManager>>) -> Result<HttpResponse, Error> {
            let conn = pool.get().expect("couldn't get db connection from pool");
            let dao = AccountDao { conn: &conn };
            let since = NaiveDate::from_ymd(2016, 10, 1).and_hms(0, 0, 0);
            let until = NaiveDate::from_ymd(2017, 10, 1).and_hms(0, 0, 0);
            let recs = dao.list();
            let accountNames = recs.into_iter().map( |a| a.name ).collect::<Vec<String>>();
            Ok(HttpResponse::Ok().json(accountNames))
        }
    }
}