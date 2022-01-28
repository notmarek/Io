table! {
    users (id) {
        id -> Varchar,
        username -> Varchar,
        password -> Varchar,
        permissions -> Array<Text>,
    }
}
