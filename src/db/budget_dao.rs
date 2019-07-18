use db::account_dao::{
    Account,
    AccountDao
};
use rest::transaction::group_by;
use chrono::{
    Datelike,
    NaiveDate
};
use postgres::Connection;
use std::collections::HashMap;

#[derive(Debug)]
#[derive(Serialize)]
pub struct Budget {
    pub guid: String,
    pub name: String,
    pub num_periods: i32,
    pub start_date: NaiveDate,
    pub amounts: Vec<BudgetAmount>
}

#[derive(Debug)]
#[derive(Serialize)]
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

        let budgets: Vec<Budget> = self.conn.query(sql, &[]).unwrap().iter().map( |row| {
            Budget {
                guid: row.get("guid"),
                name: row.get("name"),
                num_periods: row.get("num_periods"),
                start_date: row.get("recurrence_period_start"),
                amounts: Vec::new()
            }
        }).collect();

        let account_dao = AccountDao { conn: self.conn };
        let accounts = account_dao.list();

        fn get_account(accounts: &Vec<Account>, id: String) -> &Account {
            accounts.iter().find(|a| a.guid == id).unwrap()
        }

        // Don't know how to map over options and deal with lifetimes so I unwrap:(:(
        let budget = budgets.into_iter().find(|b| b.name == name).unwrap();

        let sql = "select * from budget_amounts where budget_guid = $1";

        let mut amounts = self.conn.query(&sql, &[&budget.guid]).unwrap().iter()
            .map( |row| {
                let account_guid: String = row.get("account_guid");
                let period = row.get("period_num");
                let amount: i64 = row.get("amount_num");
                let amount_denum: i64 = row.get("amount_denom");
                let multiplier = 100 / amount_denum;
                let amount_in_cents = amount * multiplier;
                BudgetAmount {
                    account: get_account(&accounts, account_guid).clone(),
                    period_num: period,
                    amount: amount_in_cents,
                    month: NaiveDate::from_ymd(budget.start_date.year(), budget.start_date.month() + (period as u32), 1) 
                }
            }).filter(|a| {
                a.account.is_expense() && a.month.year() == month.year() && a.month.month() == month.month()
            }).collect::<Vec<_>>();
            
        amounts.sort_by(|a, b| a.account.name.cmp(&b.account.name));

        Budget {
            amounts: amounts,
            ..budget
        }
    }

}

