// @generated automatically by Diesel CLI.

diesel::table! {
    heartbeats (user_id, time) {
        id -> Int4,
        time -> Timestamptz,
        created_at -> Timestamptz,
        user_id -> Int4,
        entity -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        ip_address -> Inet,
        project -> Nullable<Text>,
        branch -> Nullable<Text>,
        language -> Nullable<Text>,
        category -> Nullable<Text>,
        is_write -> Nullable<Bool>,
        editor -> Nullable<Text>,
        operating_system -> Nullable<Text>,
        machine -> Nullable<Text>,
        user_agent -> Text,
        lines -> Nullable<Int4>,
        project_root_count -> Nullable<Int4>,
        dependencies -> Nullable<Array<Nullable<Text>>>,
        line_additions -> Nullable<Int4>,
        line_deletions -> Nullable<Int4>,
        lineno -> Nullable<Int4>,
        cursorpos -> Nullable<Int4>,
        source_type -> Nullable<Text>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Int4,
        github_user_id -> Int8,
        github_access_token -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        github_id -> Int8,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 200]
        avatar_url -> Varchar,
        api_key -> Uuid,
        is_admin -> Bool,
        is_banned -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(heartbeats -> users (user_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(heartbeats, sessions, users,);
