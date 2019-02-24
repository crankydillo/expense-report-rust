#![feature(plugin)]
#![plugin(rocket_codegen)]

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime};
use db::account_dao::AccountDao;
use db::transaction_dao::{Transaction, TransactionDao, Split};
use db::account_dao::Account;
use itertools::Itertools;
use pg_conn::PgConn;
use rocket_contrib::json::Json;
use serde_json;

fn parse_nd(s: &str) -> NaiveDate {
    let with_day = |s: &str| format!("{}-01", s);
    NaiveDate::parse_from_str(&with_day(&s.replace(" ", "")), "%Y-%m-%d").unwrap()
}

#[get("/trans?<since>&<until>&<months>")]
pub fn list(
    conn: PgConn,
    since: Option<String>,
    until: Option<String>,
    months: Option<String>
) -> Json<Vec<Transaction>> {
    let dao = TransactionDao { conn: &conn };
    let (since_nd, until_nd) = since_until(since, until, months);
    let recs = dao.list(&since_nd, &until_nd);
    Json(recs)
}

#[derive(Debug)]
#[derive(Serialize)]
pub struct MonthlyTotals {
    summaries: Vec<MonthTotal>,
    totalSpent: i64,
    acctSums: Vec<MonthlyExpenseGroup>
}

#[derive(Debug)]
#[derive(Serialize)]
pub struct AccountSummary {
    name: String,
    monthlyTotals: Vec<MonthlyTotal>,
}

#[derive(Debug)]
#[derive(Serialize)]
pub struct MonthlyTotal {
    month: NaiveDate,
    total: i64
}

#[get("/monthly-totals?<since>&<until>&<months>")]
pub fn monthly_totals<'a>(
    conn: PgConn,
    since: Option<String>,
    until: Option<String>,
    months: Option<String>
) -> Json<MonthlyTotals> {
    let trans_dao = TransactionDao { conn: &conn };
    let account_dao = AccountDao { conn: &conn };
    let (since_nd, until_nd) = since_until(since, until, months);
    let trans = trans_dao.list(&since_nd, &until_nd);
    let mut accounts = account_dao.list();

    let mut unfilled_months = expenses_by_month(&trans, &accounts);

    let mut months = fill_empty_months(&since_nd, &until_nd, &unfilled_months);

    months.sort_by(|a, b| a.total.cmp(&b.total));

    let all_months = months.iter().flat_map(|m| &m.monthly_totals);

    let grouped = group_by(all_months.collect::<Vec<_>>(), |m| m.month.clone());

    let mut summed = grouped.into_iter().map(|(i, month_summary)| {
        MonthTotal {
            month: i,
            total: month_summary.into_iter().map(|m| m.total).sum()
        }
    }).collect::<Vec<_>>();

    summed.sort_by(|a, b| parse_nd(&b.month).cmp(&parse_nd(&a.month)));

    let total_spent = summed.iter().map(|m| m.total).sum();

    Json(
        MonthlyTotals {
            summaries: summed,
            totalSpent: total_spent,
            acctSums: months.clone()
        }
    )
}

fn since_until(
    since_p: Option<String>,
    until_p: Option<String>,
    months_p: Option<String>
) -> (NaiveDate, NaiveDate) {

    let until = until_p.map(|s| parse_nd(&s)).unwrap_or(Local::now().naive_local().date());
    let since = since_p.map(|s| parse_nd(&s)).unwrap_or({
        let months_since = months_p.map(|m| m.parse().unwrap()).unwrap_or(6);
        // yes I've (sort of) done the following twice, and it's crappy both times
        let mut curr_year = until.year();
        let mut curr_month = until.month();
        (0..months_since - 1).for_each(|i| {
            if curr_month == 1 {
                curr_year -= 1;
                curr_month = 12;
            } else {
                curr_month -= 1;
            };
        });
        NaiveDate::from_ymd(curr_year, curr_month, 1)
    });
    (since, until)
}

fn months_between(since: &NaiveDate, until: &NaiveDate) -> u32 {
    println!("{:?}", (since, until));

    let mut curr_year = until.year();
    let mut curr_month = until.month();
    let mut ctr = 0;

    while curr_year > since.year() || curr_year == since.year() && curr_month > since.month() {
        if curr_month == 1 {
            curr_year -= 1;
            curr_month = 12;
        } else {
            curr_month -= 1;
        };
        ctr += 1;
    }

    println!("{}", ctr);
    ctr
}

fn fill_empty_months(
    since: &NaiveDate,
    until: &NaiveDate,
    expenses: &Vec<MonthlyExpenseGroup>
) -> Vec<MonthlyExpenseGroup> {
    // don't have moment like we do in node:(
    let mut curr_year = until.year();
    let mut curr_month = until.month();
    let num_months = months_between(since, until);
    let mut desired_months = (0..num_months).map(|i| {
        if curr_month == 1 {
            curr_year -= 1;
            curr_month = 12;
        } else {
            curr_month -= 1;
        };

        NaiveDate::from_ymd(curr_year, curr_month, 1)
    }).collect::<Vec<_>>();

    desired_months.insert(0, NaiveDate::from_ymd(until.year(), until.month(), 1));

    let mut cloned_expenses = expenses.clone();
   
    (0..cloned_expenses.len()).for_each(|i| {

        let mut exp = &mut cloned_expenses[i];
        (0..num_months).for_each(|_j| {
            let j = _j as usize;
            let month_str = format_nd(desired_months[j]);
            let exp_month = exp.monthly_totals.get(j).map(|mt| mt.clone());
            if (exp_month.is_none() || month_str != exp_month.unwrap().month) {
                exp.monthly_totals.insert(j, MonthTotal {
                    month: month_str,
                    total: 0
                });
            }
        });
    });

    cloned_expenses
}

struct MonthlyExpense {
    name: String,
    date: NaiveDate,
    amount: i64,
    memo: String
}

struct ExpenseSplit {
    name: String,
    date: NaiveDateTime,
    amount: i64,
    memo: String
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(Serialize)]
struct MonthTotal {
    month: String,
    total: i64
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(Serialize)]
struct MonthlyExpenseGroup {
    name: String,
    total: i64,
    monthly_totals: Vec<MonthTotal>,
}

fn expenses_by_month(
    transactions: &Vec<Transaction>,
    accounts: &Vec<Account>
) -> Vec<MonthlyExpenseGroup>   {
    let mut accountsMap = HashMap::new();
    for a in accounts {
        accountsMap.insert(&a.guid, a);
    }

    // No need to fold/reduce here like we do in the node version.
    // That was probably just a mistake there.
    let mut splits = transactions.iter().flat_map(|tran| {
        let expenses = tran.splits.iter().filter(|s| accountsMap[&s.account_guid].is_expense()).collect::<Vec<&Split>>();

        expenses.iter().map(|e| {
            ExpenseSplit {
                name: accountsMap[&e.account_guid].qualified_name(),
                date: tran.post_date,
                amount: e.value_num,
                memo: e.memo.clone()
            }
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    splits.sort_by(|a,b| a.name.cmp(&b.name));
    let expense_groups = group_by(splits, |s| s.name.to_string());

    let expense_groups_by_month = expense_groups.into_iter().map(|(name, exp_group)| {
        let mut start = HashMap::<String, Vec<ExpenseSplit>>::new();
        let mut exp_splits = group_by(exp_group.into_iter().collect::<Vec<ExpenseSplit>>(), |item| {
            format_ndt(item.date)
        }).into_iter().collect::<Vec<_>>();

        exp_splits.sort_by(|a,b| b.0.cmp(&a.0));

        let monthly_totals = exp_splits.into_iter().map(|(month, splits)| {
            MonthTotal {
                month: month,
                total: splits.iter().map(|s| s.amount).sum()
            }
        }).collect::<Vec<_>>();

        MonthlyExpenseGroup {
            name: name.to_string(),
            total: monthly_totals.iter().map(|mt| mt.total).sum(),
            monthly_totals: monthly_totals
        }
    });

    expense_groups_by_month.collect::<Vec<_>>()
}

fn format_ndt(d: NaiveDateTime) -> String {
    format_nd(d.date())
}

fn format_nd(d: NaiveDate) -> String {
    let year =  d.year();
    let month = d.month();
    format!("{} - {:02}", year, month)
}

fn group_by<T, K : Eq + Hash>(items: Vec<T>, to_key: fn(&T) -> K) -> HashMap<K, Vec<T>> {
    let mut start: HashMap<K, Vec<T>> = HashMap::new();
    items.into_iter().for_each(|item| {
        let key = to_key(&item);
        let mut result = start.entry(key).or_insert(Vec::new());
        result.push(item);
    });
    start
}
