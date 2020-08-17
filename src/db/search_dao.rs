use rusqlite::{params, Connection};
use crate::db::transaction_dao::{Transaction, TransactionDao};

pub struct SearchDao<'a> {
    pub conn: &'a Connection,
    pub transaction_dao: &'a TransactionDao<'a>
}

impl<'a> SearchDao<'a> {

    pub fn search(&self, query: &str) -> Vec<Transaction> {
        let mut stmt = self.conn.prepare(
            "SELECT tran_id FROM search WHERE text MATCH ? ORDER BY rank"
        ).unwrap();

        let tran_ids: Vec<String> = 
            stmt.query_map(params![query], |row| {
                let id: String = row.get("tran_id").unwrap();
                Ok(id)
            })
            .unwrap().map(|r| r.unwrap()).collect();

        self.transaction_dao.trans(tran_ids)
    }

}
