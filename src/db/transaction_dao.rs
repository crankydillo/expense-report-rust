use chrono::*;
use ::serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use rusqlite::{params, Connection};

pub struct TransactionDao<'a> {
    pub conn: &'a Connection
}

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub guid: String,
    pub num: String,
    pub post_date: NaiveDateTime,
    pub description: String,
    pub splits: Vec<Split>
}

#[derive(Clone, Debug, Serialize)]
pub struct Split {
    pub account_guid: String,
    pub transaction_guid: String,
    pub value_num: i64,
    pub memo: String,
}

impl<'a> TransactionDao<'a> {

    pub fn list(
        &self,
        since: &NaiveDate,
        until: &NaiveDate,
    ) -> Vec<Transaction> {

        let since_dt = &since.and_hms(0, 0, 0).format("%Y-%m-%d %H:%M:%S").to_string();
        let until_dt = &until.and_hms(23, 59, 59).format("%Y-%m-%d %H:%M:%S").to_string();

        let mut stmt = self.conn.prepare(
            "select guid, num, post_date, description from transactions \
            where post_date >= ?1 \
            and post_date < ?2 \
            order by post_date desc"
        ).unwrap();

        let trans = stmt.query_map(params![&since_dt, &until_dt], |row| {
            Ok(Transaction {
                guid: row.get("guid").unwrap(),
                num: row.get("num").unwrap(),
                post_date: row.get("post_date").unwrap(),
                description: row.get("description").unwrap(),
                splits: Vec::new()
            })
        }).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();

        let trans_ids = trans.iter().map(|t| t.guid.clone()).collect::<Vec<_>>();

        let splits: Vec<Split> =
            if (!trans_ids.is_empty()) {
                let trans_id_query_part = trans_ids.iter().map( |id| {
                    String::from("'") + &id + &String::from("'")
                }).collect::<Vec<_>>().join(",");

                let split_query_str = String::from("select tx_guid, account_guid, memo, value_num from splits where tx_guid in (") + 
                    &trans_id_query_part + &String::from(")");

                let mut split_query = self.conn.prepare(&split_query_str).unwrap();

                split_query.query_map(params![], |row|
                    Ok(Split {
                        account_guid: row.get("account_guid").unwrap(),
                        transaction_guid: row.get("tx_guid").unwrap(),
                        value_num: row.get("value_num").unwrap(),
                        memo: row.get("memo").unwrap()
                    })
                ).unwrap().map(|r| r.unwrap()).collect()
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
