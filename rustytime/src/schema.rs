// @generated automatically by Diesel CLI.

diesel::table! {
    heartbeats (user_id, created_at) {
        id -> Int4,
        created_at -> Timestamptz,
        user_id -> Int4,
        #[max_length = 512]
        entity -> Varchar,
        #[sql_name = "type"]
        #[max_length = 50]
        type_ -> Varchar,
        ip_address -> Inet,
        #[max_length = 100]
        project -> Nullable<Varchar>,
        #[max_length = 100]
        branch -> Nullable<Varchar>,
        #[max_length = 50]
        language -> Nullable<Varchar>,
        #[max_length = 50]
        category -> Nullable<Varchar>,
        is_write -> Nullable<Bool>,
        #[max_length = 50]
        editor -> Nullable<Varchar>,
        #[max_length = 100]
        operating_system -> Nullable<Varchar>,
        #[max_length = 100]
        machine -> Nullable<Varchar>,
        #[max_length = 255]
        user_agent -> Varchar,
        lines -> Nullable<Int4>,
        project_root_count -> Nullable<Int4>,
        dependencies -> Nullable<Array<Nullable<Text>>>,
        line_additions -> Nullable<Int4>,
        line_deletions -> Nullable<Int4>,
        lineno -> Nullable<Int4>,
        cursorpos -> Nullable<Int4>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Int4,
        github_access_token -> Text,
        github_user_id -> Int8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Nullable<Varchar>,
        #[max_length = 200]
        avatar_url -> Nullable<Varchar>,
        created_at -> Timestamptz,
        api_key -> Uuid,
        github_id -> Int4,
        is_admin -> Bool,
    }
}

diesel::joinable!(heartbeats -> users (user_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(heartbeats, sessions, users,);
