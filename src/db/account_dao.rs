use postgres::Connection;
use std::collections::HashMap;

#[derive(Debug)]
#[derive(Serialize)]
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

        let db_accounts: Vec<DbAccount> = self.conn.query("SELECT * from accounts", &[]).unwrap().iter().map( |row| {
            DbAccount {
                guid: row.get("guid"),
                parent_guid: row.get("parent_guid"),
                name: row.get("name")
            }
        }).collect();


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
