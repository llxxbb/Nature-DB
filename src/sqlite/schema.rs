table! {
    instances (instance_id, thing, version, status_version) {
        instance_id -> Binary,
        thing -> Text,
        version -> Integer,
        content -> Text,
        context -> Nullable<Text>,
        status -> Nullable<Text>,
        status_version -> Integer,
        from_thing -> Nullable<Text>,
        from_version -> Nullable<Integer>,
        from_status_version -> Nullable<Integer>,
        event_time -> Timestamp,
        execute_time -> Timestamp,
        create_time -> Timestamp,
    }
}

table! {
    one_step_flow (from_thing, from_version, to_thing, to_version) {
        from_thing -> Text,
        from_version -> Integer,
        to_thing -> Text,
        to_version -> Integer,
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
        thing -> Text,
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
        thing -> Text,
        data_type -> SmallInt,
        data -> Text,
        create_time -> Timestamp,
        msg -> Text,
    }
}

table! {
    thing_defines (key, version) {
        key -> Text,
        description -> Nullable<Text>,
        version -> Integer,
        states -> Nullable<Text>,
        fields -> Nullable<Text>,
        create_time -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    instances,
    one_step_flow,
    plan,
    task,
    task_error,
    thing_defines,
);
