
table! {
    transactions (guid) {
        guid -> Text,
        num -> Text,
        post_date -> Nullable<Timestamp>,
        description -> Nullable<Text>,
    }
}

table! {
    splits (guid) {
        guid -> Text,
        #[sql_name = "tx_guid"]
        transaction_guid -> Text,
        account_guid -> Text,
        memo -> Text,
        value_num -> BigInt,
    }
}
