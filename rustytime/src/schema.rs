// @generated automatically by Diesel CLI.

diesel::table! {
    _sqlx_migrations (version) {
        version -> Int8,
        description -> Text,
        installed_on -> Timestamptz,
        success -> Bool,
        checksum -> Bytea,
        execution_time -> Int8,
    }
}

diesel::table! {
    heartbeats (user_id, time) {
        id -> Int8,
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
        source_type -> Nullable<Int2>,
        project_id -> Nullable<Int4>,
    }
}

diesel::table! {
    import_jobs (id) {
        id -> Int8,
        user_id -> Int4,
        #[max_length = 20]
        status -> Varchar,
        imported_count -> Nullable<Int8>,
        processed_count -> Nullable<Int8>,
        request_count -> Nullable<Int4>,
        start_date -> Nullable<Text>,
        time_taken -> Nullable<Float8>,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    leaderboards (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 20]
        period_type -> Varchar,
        period_date -> Date,
        total_seconds -> Int8,
        rank -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    project_alias_resolutions (user_id, project_id) {
        user_id -> Int4,
        project_id -> Int4,
        resolved_project_id -> Int4,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    project_aliases (id) {
        id -> Int4,
        user_id -> Int4,
        project_id -> Int4,
        alias_to -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    projects (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Text,
        repo_url -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        hidden -> Bool,
        project_url -> Nullable<Text>,
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
        impersonated_by -> Nullable<Int4>,
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
        admin_level -> Int2,
        is_banned -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 50]
        timezone -> Varchar,
    }
}

diesel::joinable!(heartbeats -> users (user_id));
diesel::joinable!(import_jobs -> users (user_id));
diesel::joinable!(leaderboards -> users (user_id));
diesel::joinable!(project_alias_resolutions -> users (user_id));
diesel::joinable!(project_aliases -> users (user_id));
diesel::joinable!(projects -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    _sqlx_migrations,
    heartbeats,
    import_jobs,
    leaderboards,
    project_alias_resolutions,
    project_aliases,
    projects,
    sessions,
    users,
);
