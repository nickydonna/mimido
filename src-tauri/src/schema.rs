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
        ical_data -> Text,
        last_modified -> BigInt,
        summary -> Text,
        href -> Text,
        ends_at -> TimestamptzSqlite,
        description -> Nullable<Text>,
        starts_at -> TimestamptzSqlite,
        event_type -> Text,
        tag -> Nullable<Text>,
        status -> Text,
        original_text -> Nullable<Text>,
        importance -> Integer,
        load -> Integer,
        urgency -> Integer,
        postponed -> Integer,
        has_rrule -> Bool,
        rrule_str -> Nullable<Text>,
        etag -> Text,
    }
}

diesel::table! {
    vtodos (id) {
        id -> Integer,
        calendar_id -> Integer,
        uid -> Text,
        href -> Text,
        ical_data -> Text,
        last_modified -> BigInt,
        summary -> Text,
        description -> Nullable<Text>,
        event_type -> Text,
        tag -> Nullable<Text>,
        status -> Text,
        original_text -> Nullable<Text>,
        importance -> Integer,
        load -> Integer,
        urgency -> Integer,
        postponed -> Integer,
        etag -> Text,
        has_rrule -> Bool,
        rrule_str -> Nullable<Text>,
        starts_at -> Nullable<TimestamptzSqlite>,
        ends_at -> Nullable<TimestamptzSqlite>,
        completed -> Nullable<TimestamptzSqlite>,
    }
}

diesel::joinable!(calendars -> servers (server_id));
diesel::joinable!(vevents -> calendars (calendar_id));
diesel::joinable!(vtodos -> calendars (calendar_id));

diesel::allow_tables_to_appear_in_same_query!(calendars, servers, vevents, vtodos,);
