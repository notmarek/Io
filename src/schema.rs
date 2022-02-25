table! {
    libraries (id) {
        id -> Int4,
        path -> Varchar,
        depth -> Int4,
    }
}

table! {
    users (id) {
        id -> Varchar,
        username -> Varchar,
        password -> Varchar,
        permissions -> Array<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    libraries,
    users,
);
