use chrono::*;
use std::collections::HashMap;
use ::serde::Serialize;
use rusqlite::{params, Connection};

use crate::db::account_dao::{
    Account,
    AccountDao
};

#[derive(Debug, Serialize)]
pub struct Budget {
    pub guid: String,
    pub name: String,
    pub num_periods: i32,
    pub start_date: NaiveDate,
    pub amounts: Vec<BudgetAmount>
}

#[derive(Debug, Serialize)]
pub struct BudgetAmount {
    pub account: Account,
    pub period_num: i32,
    pub amount: i64,
    pub month: NaiveDate
}

pub struct BudgetDao<'a> {
    pub conn: &'a Connection,
}

impl<'a> BudgetDao<'a> {

    pub fn get_curr(
        &self,
        month: NaiveDate
    ) -> Option<Budget> {
        // TODO, fix the crappy duplication and modeling!
        let sql = "select * from budgets b, recurrences r where b.guid = r.obj_guid";
        let mut stmt = self.conn.prepare(sql).unwrap();

        let budgets = stmt.query_map(params![], |row| {
            let date_str: String = row.get("recurrence_period_start").unwrap();
            Ok(Budget {
                guid: row.get("guid").unwrap(),
                name: row.get("name").unwrap(),
                num_periods: row.get("num_periods").unwrap(),
                start_date: NaiveDate::parse_from_str(&date_str, "%Y%m%d").unwrap(),
                amounts: Vec::new()
            })
        }).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();
;

        let budgets_with_dates = budgets.into_iter().map(|b| {
            let mut curr_year = b.start_date.year();
            let mut curr_month = b.start_date.month();
            let mut dates = vec!(b.start_date);
            let months = (0..b.num_periods-1).for_each(|i| {
                if curr_month == 12 {
                    curr_year += 1;
                    curr_month = 1;
                } else {
                    curr_month += 1;
                };
                dates.push(NaiveDate::from_ymd(curr_year, curr_month, 1));
            });

            (b, dates)
        });

        budgets_with_dates.into_iter().find(|(_, dates)| {
            dates.into_iter().find(|d| d.year() == month.year() && d.month() == month.month()).is_some()
        }).map(|(b, _)| self.get(b.name, month))
    }

    // TODO Return Optional<Budget>
    pub fn get(
        &self,
        name: String,
        month: NaiveDate
    ) -> Budget {

        struct DbAccount {
            guid: String,
            parent_guid: Option<String>,
            name: String
        }

        let sql = "select * from budgets b, recurrences r where b.guid = r.obj_guid";
        let mut stmt = self.conn.prepare(sql).unwrap();

        let budgets = stmt.query_map(params![], |row| {
            Ok(Budget {
                guid: row.get("guid").unwrap(),
                name: row.get("name").unwrap(),
                num_periods: row.get("num_periods").unwrap(),
                start_date: row.get("recurrence_period_start").unwrap(),
                amounts: Vec::new()
            })
        }).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();

        let account_dao = AccountDao { conn: self.conn };
        let accounts = account_dao.list();

        fn get_account(accounts: &Vec<Account>, id: String) -> &Account {
            accounts.iter().find(|a| a.guid == id).unwrap()
        }

        // Don't know how to map over options and deal with lifetimes so I unwrap:(:(
        let budget = budgets.into_iter().find(|b| b.name == name).unwrap();

        let ba_query = "select * from budget_amounts where budget_guid = ?1";
        let mut ba_stmt = self.conn.prepare(ba_query).unwrap();

        let mut amounts = ba_stmt.query_map(params![&budget.guid], |row| {
            let account_guid: String = row.get("account_guid").unwrap();
            let period = row.get("period_num").unwrap();
            let amount: i64 = row.get("amount_num").unwrap();
            let amount_denum: i64 = row.get("amount_denom").unwrap();
            let multiplier = 100 / amount_denum;
            let amount_in_cents = amount * multiplier;
            Ok(BudgetAmount {
                account: get_account(&accounts, account_guid).clone(),
                period_num: period,
                amount: amount_in_cents,
                month: NaiveDate::from_ymd(budget.start_date.year(), budget.start_date.month() + (period as u32), 1) 
            })
        }).unwrap()
        .map(|r| r.unwrap())
        .filter(|a| {
            a.account.is_expense() && a.month.year() == month.year() && a.month.month() == month.month()
        }).collect::<Vec<_>>();

        amounts.sort_by(|a, b| a.account.name.cmp(&b.account.name));

        Budget {
            amounts: amounts,
            ..budget
        }
    }
}
