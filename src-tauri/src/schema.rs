// @generated automatically by Diesel CLI.

diesel::table! {
    calendars (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
        etag -> Nullable<Text>,
        server_id -> Integer,
    }
}

diesel::table! {
    events (id) {
        id -> Integer,
        calendar_id -> Integer,
        uid -> Text,
        ical_data -> Text,
        last_modified -> BigInt,
        summary -> Text,
        href -> Text,
        ends_at -> TimestamptzSqlite,
        recur -> Nullable<Text>,
        description -> Nullable<Text>,
        starts_at -> TimestamptzSqlite,
    }
}

diesel::table! {
    servers (id) {
        server_url -> Text,
        user -> Text,
        password -> Text,
        id -> Integer,
        last_sync -> Nullable<BigInt>,
    }
}

diesel::table! {
    todo_lists (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
        ctag -> Text,
    }
}

diesel::table! {
    todos (id) {
        id -> Integer,
        list_id -> Integer,
        uid -> Text,
        etag -> Text,
        url -> Text,
        ical_data -> Text,
        last_modified -> BigInt,
        completed -> Bool,
    }
}

diesel::joinable!(calendars -> servers (server_id));
diesel::joinable!(events -> calendars (calendar_id));
diesel::joinable!(todos -> todo_lists (list_id));

diesel::allow_tables_to_appear_in_same_query!(calendars, events, servers, todo_lists, todos,);
