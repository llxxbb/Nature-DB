table! {
    instances (ins_key, state_version) {
        ins_key -> Varchar,
        content -> Text,
        context -> Nullable<Text>,
        states -> Nullable<Text>,
        state_version -> Integer,
        create_time -> Datetime,
        sys_context -> Nullable<Text>,
        from_key -> Varchar,
    }
}

table! {
    meta (meta_type, meta_key, version) {
        meta_type -> Varchar,
        meta_key -> Varchar,
        description -> Nullable<Varchar>,
        version -> Integer,
        states -> Nullable<Varchar>,
        fields -> Nullable<Varchar>,
        config -> Varchar,
        flag -> Integer,
        create_time -> Datetime,
    }
}

table! {
    relation (from_meta, to_meta) {
        from_meta -> Varchar,
        to_meta -> Varchar,
        settings -> Varchar,
        flag -> Integer,
    }
}

table! {
    task (task_id) {
        task_id -> Binary,
        task_key -> Varchar,
        task_type -> Tinyint,
        task_for -> Varchar,
        task_state -> Tinyint,
        data -> Text,
        create_time -> Datetime,
        execute_time -> Datetime,
        retried_times -> Smallint,
    }
}

table! {
    task_error (task_id) {
        task_id -> Binary,
        task_key -> Varchar,
        task_type -> Tinyint,
        task_for -> Varchar,
        data -> Text,
        create_time -> Datetime,
        msg -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    instances,
    meta,
    relation,
    task,
    task_error,
);
