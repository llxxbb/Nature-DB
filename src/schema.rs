table! {
    instances (meta, para, instance_id, state_version) {
        instance_id -> Binary,
        meta -> Varchar,
        para -> Varchar,
        content -> Varchar,
        context -> Nullable<Text>,
        states -> Nullable<Text>,
        state_version -> Integer,
        from_meta -> Nullable<Varchar>,
        from_para -> Nullable<Varchar>,
        from_id -> Nullable<Binary>,
        from_state_version -> Nullable<Integer>,
        execute_time -> Datetime,
        create_time -> Datetime,
        sys_context -> Nullable<Text>,
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
    plan (upstream, downstream) {
        upstream -> Varchar,
        downstream -> Varchar,
        content -> Text,
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
        meta -> Varchar,
        data_type -> Smallint,
        data -> Text,
        last_state_version -> Integer,
        create_time -> Datetime,
        execute_time -> Datetime,
        retried_times -> Smallint,
    }
}

table! {
    task_error (task_id) {
        task_id -> Binary,
        meta -> Varchar,
        data_type -> Smallint,
        data -> Text,
        last_state_version -> Integer,
        create_time -> Datetime,
        msg -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    instances,
    meta,
    plan,
    relation,
    task,
    task_error,
);
