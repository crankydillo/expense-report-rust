#![feature(plugin)]
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

use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

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
        .mount("/res/", routes![
               routes::account::list,
               routes::transaction::list,
               routes::transaction::monthly_totals,
               routes::transaction::expense_splits,
               routes::budget::get
        ])
        .mount("/", StaticFiles::from("static"))
        .manage(conn_str)
        .manage(pool)
        .launch();
}

mod routes {

    pub mod budget {
        use chrono::Local;
        use db::pg_conn::PgConn;
        use db::budget_dao::BudgetDao;
        use rest::transaction;
        use rocket_contrib::json::Json;
        use std::collections::HashSet;
        use std::iter::FromIterator;

        #[get("/budget")]
        pub fn get(
            conn: PgConn
        ) -> Json<Budget> {
            let budget_dao = BudgetDao { conn: &conn };
            let now = Local::now().naive_local().date();
            let budget = budget_dao.get("Summer 2019".to_string(), now);
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

            Json(Budget {
                name: budget.name,
                amounts: budget_amounts
            })
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
            conn: PgConn,
            since: Option<String>,
            until: Option<String>,
            months: Option<String>,
            year: Option<String>
        ) -> Json<transaction::MonthlyTotals> {
            Json(transaction::monthly_totals(&conn, since, until, months, year))
        }

        #[get("/trans?<since>&<until>&<months>&<year>")]
        pub fn list(
            conn: PgConn,
            since: Option<String>,
            until: Option<String>,
            months: Option<String>,
            year: Option<String>
        ) -> Json<Vec<Transaction>> {
            Json(transaction::list(conn, since, until, months, year))
        }

        #[get("/expenses/<expense_name>/<month>")]
        pub fn expense_splits<'a>(
            conn: PgConn,
            expense_name: String,
            month: String
        ) -> Json<ExpenseSplits> {
            let splits = transaction::expense_splits(conn, expense_name, month);
            let exp_splits = ExpenseSplits {
                msg: "hi!".to_string(),
                splits: splits
            };
            Json(exp_splits)
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
        pub fn list(conn: PgConn) -> Json<Vec<String>> {
            let dao = AccountDao { conn: &conn };
            let since = NaiveDate::from_ymd(2016, 10, 1).and_hms(0, 0, 0);
            let until = NaiveDate::from_ymd(2017, 10, 1).and_hms(0, 0, 0);
            let recs = dao.list();
            let accountNames = recs.into_iter().map( |a| a.name ).collect::<Vec<String>>();
            Json(accountNames)
        }
    }
}
