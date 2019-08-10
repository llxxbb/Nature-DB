table! {
    instances (instance_id, meta, para, status_version) {
        instance_id -> Binary,
        meta -> Text,
        para -> Text,
        content -> Text,
        context -> Nullable<Text>,
        status -> Nullable<Text>,
        status_version -> Integer,
        from_meta -> Nullable<Text>,
        from_status_version -> Nullable<Integer>,
        event_time -> Timestamp,
        execute_time -> Timestamp,
        create_time -> Timestamp,
    }
}

table! {
    meta (full_key, version) {
        full_key -> Text,
        description -> Nullable<Text>,
        version -> Integer,
        states -> Nullable<Text>,
        fields -> Nullable<Text>,
        create_time -> Timestamp,
    }
}

table! {
    one_step_flow (from_meta, to_meta) {
        from_meta -> Text,
        to_meta -> Text,
        settings -> Text,
    }
}

table! {
    plan (upstream, to_biz, to_version) {
        upstream -> Text,
        to_biz -> Text,
        to_version -> Integer,
        content -> Text,
        create_time -> Timestamp,
    }
}

table! {
    task (task_id) {
        task_id -> Binary,
        meta -> Text,
        data_type -> SmallInt,
        data -> Text,
        create_time -> Timestamp,
        execute_time -> Timestamp,
        retried_times -> SmallInt,
    }
}

table! {
    task_error (task_id) {
        task_id -> Binary,
        meta -> Text,
        data_type -> SmallInt,
        data -> Text,
        create_time -> Timestamp,
        msg -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    instances,
    meta,
    one_step_flow,
    plan,
    task,
    task_error,
);
