table! {
    users (id) {
        id -> Int4,
        email -> Nullable<Varchar>,
        viber_id -> Nullable<Varchar>,
        broadcast -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(users,);
