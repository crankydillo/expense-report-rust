use chrono::*;
use diesel::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;

use crate::models::*;
use crate::schema::transactions::dsl::*;
use crate::schema::splits::dsl::*;

pub struct TransactionDao<'a> {
    pub conn: &'a SqliteConnection
}

impl<'a> TransactionDao<'a> {

    pub fn list(
        &self,
//        since: &NaiveDate,
 //       until: &NaiveDate,
    ) -> Vec<Transaction2> {

        let trans = transactions
            .limit(50)
            .load::<Transaction>(self.conn)
            .expect("Error loading transactions");

        let trans_ids = trans.iter().map(|t| t.guid.clone()).collect::<Vec<_>>();

        let _splits = self.splits(trans_ids);
        let mut splits_by_tran: HashMap<String, Vec<Split>> = HashMap::new();

        _splits.into_iter().for_each(|s| {
            let trans_id = s.transaction_guid.clone();
            let mut tracked_splits = match splits_by_tran.remove(&s.transaction_guid) {
                Some(mut ss) => ss,
                _        => Vec::new()
            };
            tracked_splits.push(s);
            splits_by_tran.insert(trans_id, tracked_splits);
        });

        trans.into_iter().map(|t| {
            Transaction2 {
                guid: t.guid.clone(),
                num: t.num,
                post_date: t.post_date,
                description: t.description,
                splits: splits_by_tran.remove(&t.guid).unwrap_or(Vec::new()),
            }
        }).collect()
    }

    fn splits(
        &self,
        tran_ids: Vec<String>,
    ) -> Vec<Split> {
        if (!tran_ids.is_empty()) {

            let trans_id_query_part = tran_ids.iter().map( |id| {
                String::from("'") + &id + &String::from("'")
            }).collect::<Vec<_>>().join(",");

            let split_query = String::from("select guid, tx_guid as transaction_guid, \
                                               account_guid, memo, value_num from splits \
                                               where tx_guid in (") + 
                              &trans_id_query_part + &String::from(")");


            diesel::sql_query(split_query)
                .load(self.conn)
                .expect("splits")

        } else {
            Vec::new()  // no constant Vec::empty()?  Does that make sense with no GC?
        }
    }
}
