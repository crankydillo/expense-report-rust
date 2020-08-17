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
        .max_size(5)
        .build(manager)
        .expect("Failed to create pool.");

    tokio::spawn(search::index_transactions(pool.clone()));

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(get)
            .service(routes::budget::get)
            .service(routes::transaction::monthly_totals)
            .service(routes::transaction::list)
            .service(routes::transaction::expense_splits)
            .service(routes::account::list)
            .service(routes::search::search)
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .workers(1)
    .bind("0.0.0.0:8088")?
    .run()
    .await
}

#[actix_web::get("/hi")]
pub async fn get() -> impl actix_web::Responder {
    "Hello"
}
 
mod routes {

    pub mod search {
        use chrono::NaiveDate;
        use actix_web::{get, web, App, Error, HttpResponse, HttpServer};
        use actix_web::web::Query;
        use ::serde::{Deserialize, Serialize};
        use r2d2::Pool;
        use r2d2_sqlite::SqliteConnectionManager;
        use crate::db::account_dao::AccountDao;
        use crate::db::transaction_dao::TransactionDao;
        use crate::db::search_dao::SearchDao;
        use std::collections::HashMap;

        #[derive(Debug)]
        #[derive(Serialize)]
        pub struct SearchTran {
            pub date: NaiveDate,
            pub description: String,
            pub memo: String,
            pub amount: i64
        }

        #[derive(Deserialize)]
        struct SearchParams {
            q: String
        }

        #[get("/res/search")]
        pub async fn search(
            pool: web::Data<Pool<SqliteConnectionManager>>,
            params: Query<SearchParams>
        ) -> Result<HttpResponse, Error> {
            let conn = pool.get().expect("couldn't get db connection from pool");
            let accounts = AccountDao { conn: &conn }.list();
            let mut accounts_map = HashMap::new();

            for a in &accounts {
                accounts_map.insert(&a.guid, a);
            }

            let tran_dao = TransactionDao { conn: &conn };

            let search_dao = SearchDao {
                conn: &conn,
                transaction_dao: &tran_dao
            };
            let results =
                search_dao.search(&params.q).into_iter().map(|t| {

                    let amount: i64 =
                        t.splits.iter()
                        .filter(|s| accounts_map[&s.account_guid].is_expense())
                        .map(|s| s.value_num)
                        .sum();

                    let memo =
                        t.splits.iter()
                            .map(|s| &*s.memo)
                            .collect::<Vec<&str>>().join(" ");

                    SearchTran {
                        date: t.post_date.date(),
                        description: t.description,
                        memo: memo,
                        amount: amount
                    }
                }).collect::<Vec<_>>();
            Ok(HttpResponse::Ok().json(results))
        }
    }

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
            let recs = dao.list();
            let accountNames = recs.into_iter().map( |a| a.name ).collect::<Vec<String>>();
            Ok(HttpResponse::Ok().json(accountNames))
        }
    }
}

mod search {
    use std::time::{Duration, Instant};
    use tokio::time;

    use std::collections::HashMap;

    use crate::db::*;
    use crate::db::account_dao::AccountDao;
    use crate::db::transaction_dao::*;
    use crate::rest::transaction;

    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use rusqlite::{params, Connection};

    pub async fn index_transactions(pool: Pool<SqliteConnectionManager>) {
        let mut interval_day = time::interval(Duration::from_secs(1 * 24 * 60 * 60));
        interval_day.tick().await; // we don't want to start indexing at app startup!
        loop {
            let now = interval_day.tick().await;
            println!("Indexing started at {:?}", Instant::now());
            let conn = pool.get().expect("couldn't get db connection from pool");

            let tots = transaction::list(
                &conn,
                Some("2001-01".to_string()),
                Some("2099-01".to_string()),
                None,
                None
            );

            let accounts = AccountDao { conn: &conn }.list();
            let mut accounts_map = HashMap::new();

            conn.execute("DROP TABLE search", params![]);

            conn.execute(
                "CREATE VIRTUAL TABLE search USING FTS5(tran_id, text, tokenize = porter)",
                params![]
            ).unwrap();

            for a in &accounts {
                accounts_map.insert(&a.guid, a);
            }

            let mut insert_search_statement = conn.prepare(
                "INSERT INTO search VALUES (?, ?)"
            ).unwrap();

            println!("Indexing {} transactions.", tots.len());

            for (pos, t) in tots.iter().enumerate() {
                let amount: i64 =
                    t.splits.iter()
                    .filter(|s| accounts_map[&s.account_guid].is_expense())
                    .map(|s| s.value_num)
                    .sum();

                if (amount != 0) {
                    let split_text = t.splits.iter().map(|s| &*s.memo).collect::<Vec<&str>>().join(" ");
                    let text = format!("{} {}", t.description, split_text);
                    insert_search_statement.execute(&[&t.guid, &text]).unwrap();
                }
            }

            println!("Indexing finished at {:?}", Instant::now());
        }
    }
}
