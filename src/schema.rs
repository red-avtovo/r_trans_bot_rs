table! {
    dirs (id) {
        id -> Uuid,
        user_id -> Int8,
        alias -> Varchar,
        path -> Varchar,
        ordinal -> Int4,
        created_at -> Nullable<Timestamptz>,
    }
}

table! {
    magnets (id) {
        id -> Uuid,
        user_id -> Nullable<Int8>,
        url -> Varchar,
        created_at -> Nullable<Timestamptz>,
    }
}

table! {
    r_users (id) {
        id -> Int8,
        chat -> Int8,
        first_name -> Varchar,
        last_name -> Nullable<Varchar>,
        username -> Nullable<Varchar>,
        salt -> Varchar,
        created_at -> Nullable<Timestamptz>,
    }
}

table! {
    servers (id) {
        id -> Uuid,
        user_id -> Nullable<Int8>,
        url -> Varchar,
        username -> Nullable<Varchar>,
        password -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
    }
}

table! {
    tasks (id) {
        id -> Uuid,
        user_id -> Int8,
        server_id -> Uuid,
        magnet_id -> Uuid,
        status -> Task_status,
        description -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

table! {
    users (id) {
        id -> Int8,
        chat -> Int8,
        first_name -> Varchar,
        last_name -> Nullable<Varchar>,
        username -> Nullable<Varchar>,
        salt -> Varchar,
        created_at -> Nullable<Timestamptz>,
    }
}

joinable!(dirs -> users (user_id));
joinable!(magnets -> users (user_id));
joinable!(servers -> users (user_id));
joinable!(tasks -> magnets (magnet_id));
joinable!(tasks -> servers (server_id));
joinable!(tasks -> users (user_id));

allow_tables_to_appear_in_same_query!(
    dirs,
    magnets,
    r_users,
    servers,
    tasks,
    users,
);
