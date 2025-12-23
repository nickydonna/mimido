// @generated automatically by Diesel CLI.

diesel::table! {
    calendars (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
        etag -> Nullable<Text>,
        server_id -> Integer,
        default_value -> Bool,
        is_default -> Bool,
        sync_token -> Nullable<Text>,
        synced_at -> Nullable<TimestamptzSqlite>,
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
    vevents (id) {
        id -> Integer,
        calendar_id -> Integer,
        uid -> Text,
        href -> Nullable<Text>,
        ical_data -> Nullable<Text>,
        summary -> Text,
        description -> Nullable<Text>,
        starts_at -> TimestamptzSqlite,
        ends_at -> TimestamptzSqlite,
        has_rrule -> Bool,
        rrule_str -> Nullable<Text>,
        tag -> Nullable<Text>,
        status -> Text,
        event_type -> Text,
        original_text -> Nullable<Text>,
        load -> Integer,
        urgency -> Integer,
        importance -> Integer,
        postponed -> Integer,
        last_modified -> BigInt,
        etag -> Nullable<Text>,
        synced_at -> BigInt,
    }
}

diesel::table! {
    vtodos (id) {
        id -> Integer,
        calendar_id -> Integer,
        uid -> Text,
        href -> Nullable<Text>,
        ical_data -> Nullable<Text>,
        summary -> Text,
        description -> Nullable<Text>,
        starts_at -> Nullable<TimestamptzSqlite>,
        ends_at -> Nullable<TimestamptzSqlite>,
        has_rrule -> Bool,
        rrule_str -> Nullable<Text>,
        tag -> Nullable<Text>,
        status -> Text,
        event_type -> Text,
        original_text -> Nullable<Text>,
        load -> Integer,
        urgency -> Integer,
        importance -> Integer,
        postponed -> Integer,
        last_modified -> BigInt,
        etag -> Nullable<Text>,
        synced_at -> BigInt,
        completed -> Nullable<TimestamptzSqlite>,
    }
}

diesel::joinable!(calendars -> servers (server_id));
diesel::joinable!(vevents -> calendars (calendar_id));
diesel::joinable!(vtodos -> calendars (calendar_id));

diesel::allow_tables_to_appear_in_same_query!(calendars, servers, vevents, vtodos,);
