table! {
    config (id) {
        id -> Uuid,
        name -> Text,
        path -> Text,
    }
}

table! {
    endpoint (id) {
        id -> Uuid,
        name -> Text,
        description -> Text,
        url -> Text,
    }
}

table! {
    instruction (id) {
        id -> Uuid,
        endpoint -> Uuid,
        #[sql_name = "type"]
        type_ -> Int2,
        sequence -> Int4,
    }
}

table! {
    instruction_call_module (instruction_id) {
        instruction_id -> Uuid,
        config -> Uuid,
        method -> Text,
        out_variable_name -> Text,
    }
}

table! {
    instruction_call_module_parameter (instruction_call_module_id, sequence) {
        instruction_call_module_id -> Uuid,
        sequence -> Int2,
        name -> Text,
        arg_type -> Int2,
        arg_type_value -> Text,
    }
}

table! {
    instruction_json_return (instruction_id) {
        instruction_id -> Uuid,
        arg_type -> Int2,
        arg_type_value -> Text,
    }
}

joinable!(instruction -> endpoint (endpoint));
joinable!(instruction_call_module -> config (config));
joinable!(instruction_call_module -> instruction (instruction_id));
joinable!(instruction_call_module_parameter -> instruction_call_module (instruction_call_module_id));
joinable!(instruction_json_return -> instruction (instruction_id));

allow_tables_to_appear_in_same_query!(
    config,
    endpoint,
    instruction,
    instruction_call_module,
    instruction_call_module_parameter,
    instruction_json_return,
);
