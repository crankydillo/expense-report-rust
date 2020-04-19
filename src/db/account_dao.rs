use std::collections::HashMap;
use ::serde::Serialize;
use rusqlite::{params, Connection};

#[derive(Clone, Debug, Serialize)]
pub struct Account {
    pub guid: String,
    pub parent: Option<Box<Account>>,
    pub name: String
}

impl Account {
    pub fn is_expense(&self) -> bool {
        match self.parent {
            None => false,
            Some(ref p) if p.name.to_lowercase() == "expenses" => true,
            Some(ref p) => p.is_expense()
        }
    } 

    pub fn qualified_name(&self) -> String {
        match self.parent {
            None => self.name.to_string(),
            Some(ref p) => format!("{}:{}", p.qualified_name(), self.name)
        }
    }
}

pub struct AccountDao<'a> {
    pub conn: &'a Connection
}

impl<'a> AccountDao<'a> {

    pub fn list(&self) -> Vec<Account> {

        struct DbAccount {
            guid: String,
            parent_guid: Option<String>,
            name: String
        }

        let mut stmt = self.conn.prepare("SELECT * from accounts").unwrap();

        let db_accounts = stmt.query_map(params![], |row| {
            Ok(DbAccount {
                guid: row.get("guid").unwrap(),
                parent_guid: row.get("parent_guid").unwrap(),
                name: row.get("name").unwrap()
            })
        }).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();


        fn to_acct(db_acct: &DbAccount, by_guid: &HashMap<String, &DbAccount>) -> Account {
            let parent_acct =
                match db_acct.parent_guid {
                    Some(ref p) => Some(Box::new(to_acct(by_guid.get(p).unwrap(), by_guid))),
                    _           => None

                };
            Account {
                guid: db_acct.guid.clone(),
                parent: parent_acct,
                name: db_acct.name.clone()
            }
        };

        let by_guid: HashMap<String, &DbAccount> =
            db_accounts.iter().map( |a| (a.guid.clone(), a) ).collect();

        db_accounts.iter().map(|da| to_acct(da, &by_guid)).collect()
    }

}
