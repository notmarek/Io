table! {
    files (id) {
        id -> Varchar,
        parent -> Varchar,
        library_id -> Varchar,
        path -> Varchar,
        folder -> Bool,
        last_update -> Int8,
        title -> Nullable<Varchar>,
        season -> Nullable<Varchar>,
        episode -> Nullable<Float4>,
        release_group -> Nullable<Varchar>,
    }
}

table! {
    libraries (id) {
        id -> Varchar,
        path -> Varchar,
        depth -> Int4,
        last_scan -> Int4,
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
    files,
    libraries,
    users,
);
