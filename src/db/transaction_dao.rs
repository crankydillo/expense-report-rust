use chrono::*;
use postgres::Connection;
use postgres::stmt::Statement;
use std::collections::HashMap;

#[derive(Debug)]
#[derive(Serialize)]
pub struct Transaction {
    pub guid: String,
    pub num: String,
    pub post_date: NaiveDateTime,
    pub description: String,
    pub splits: Vec<Split>
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Serialize)]
pub struct Split {
    pub account_guid: String,
    pub transaction_guid: String,
    pub value_num: i64,
    pub memo: String,
}

pub struct TransactionDao<'a> {
    pub conn: &'a Connection
}

impl<'a> TransactionDao<'a> {

    pub fn list(
        &self,
        since: &NaiveDate,
        until: &NaiveDate,
    ) -> Vec<Transaction> {

        let since_dt = &since.and_hms(0, 0, 0);
        let until_dt = &until.and_hms(0, 0, 0);

        let query = 
            "select guid, num, post_date, description from transactions \
            where post_date >= $1 \
            and post_date < $2 \
            order by post_date desc";

        let trans = self.conn.query(&query, &[since_dt, until_dt]).unwrap().iter().map( |row| {
            Transaction {
                guid: row.get("guid"),
                num: row.get("num"),
                post_date: row.get("post_date"),
                description: row.get("description"),
                splits: Vec::new()
            }
        }).collect::<Vec<_>>();

        let trans_ids = trans.iter().map(|t| t.guid.clone()).collect::<Vec<_>>();


        let splits: Vec<Split> =
            if (!trans_ids.is_empty()) {
                let trans_id_query_part = trans_ids.iter().map( |id| {
                    String::from("'") + &id + &String::from("'")
                }).collect::<Vec<_>>().join(",");

                let split_query = String::from("select tx_guid, account_guid, memo, value_num from splits where tx_guid in (") + 
                    &trans_id_query_part + &String::from(")");

                self.conn.query(&split_query, &[]).unwrap().iter().map( |row| {
                    Split {
                        account_guid: row.get("account_guid"),
                        transaction_guid: row.get("tx_guid"),
                        value_num: row.get("value_num"),
                        memo: row.get("memo")
                    }
                }).collect()
            } else {
                Vec::new()  // no constant Vec::empty()?  Does that make sense with no GC?
            };

        let mut splits_by_tran: HashMap<String, Vec<Split>> = HashMap::new();

        splits.into_iter().for_each(|s| {
            let trans_id = s.transaction_guid.clone();
            let mut tracked_splits = match splits_by_tran.remove(&s.transaction_guid) {
                Some(mut ss) => ss,
                _        => Vec::new()
            };
            tracked_splits.push(s);
            splits_by_tran.insert(trans_id, tracked_splits);
        });

        trans.into_iter().map(|t| {
            Transaction {
                splits: splits_by_tran.remove(&t.guid).unwrap_or(Vec::new()),
                ..t
            }
        }).collect()
    }

    // TODO
    fn date_fmt(&self, d: &str) -> String {
        "abc".to_string()
    }
}
