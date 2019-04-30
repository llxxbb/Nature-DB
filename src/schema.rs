table! {
    instances (instance_id, thing, version, status_version) {
        instance_id -> Binary,
        thing -> Varchar,
        version -> Integer,
        content -> Varchar,
        context -> Nullable<Text>,
        status -> Nullable<Text>,
        status_version -> Integer,
        from_thing -> Nullable<Varchar>,
        from_version -> Nullable<Integer>,
        from_status_version -> Nullable<Integer>,
        event_time -> Datetime,
        execute_time -> Datetime,
        create_time -> Datetime,
    }
}

table! {
    one_step_flow (from_thing, from_version, to_thing, to_version) {
        from_thing -> Varchar,
        from_version -> Integer,
        to_thing -> Varchar,
        to_version -> Integer,
        settings -> Varchar,
    }
}

table! {
    plan (upstream, to_biz, to_version) {
        upstream -> Varchar,
        to_biz -> Varchar,
        to_version -> Integer,
        content -> Text,
        create_time -> Datetime,
    }
}

table! {
    task (task_id) {
        task_id -> Binary,
        thing -> Varchar,
        data_type -> Smallint,
        data -> Text,
        create_time -> Datetime,
        execute_time -> Datetime,
        retried_times -> Smallint,
    }
}

table! {
    task_error (task_id) {
        task_id -> Binary,
        thing -> Varchar,
        data_type -> Smallint,
        data -> Text,
        create_time -> Datetime,
        msg -> Varchar,
    }
}

table! {
    thing_defines (key, version) {
        key -> Varchar,
        description -> Nullable<Varchar>,
        version -> Integer,
        states -> Nullable<Varchar>,
        fields -> Nullable<Varchar>,
        create_time -> Datetime,
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
