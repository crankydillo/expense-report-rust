use crate::schema::splits;
use chrono::*;
use ::serde::Serialize;



#[derive(Debug, Serialize)]
pub struct Transaction2 {
    pub guid: String,
    pub num: String,
    pub post_date: Option<NaiveDateTime>,
    pub description: Option<String>,
    pub splits: Vec<Split>,
}

#[derive(diesel::Queryable, Debug, Serialize)]
pub struct Transaction {
    pub guid: String,
    pub num: String,
    pub post_date: Option<NaiveDateTime>,
    pub description: Option<String>,
}

#[derive(diesel::QueryableByName, Debug, Clone, Serialize)]
#[table_name = "splits"]
pub struct Split {
    pub guid: String,
    pub account_guid: String,
    pub transaction_guid: String,
    pub memo: String,
    pub value_num: i64,
}
