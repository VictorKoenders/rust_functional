-- Your SQL goes here
CREATE TABLE config (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    name TEXT NOT NULL UNIQUE,
    path TEXT NOT NULL
);

CREATE TABLE endpoint (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL UNIQUE
);

CREATE TABLE instruction (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    endpoint UUID NOT NULL REFERENCES endpoint(id),
    type SMALLINT NOT NULL,
    sequence INT NOT NULL,

    UNIQUE(endpoint, sequence)
);

CREATE TABLE instruction_json_return (
    instruction_id UUID NOT NULL REFERENCES instruction(id),
    arg_type SMALLINT NOT NULL,
    arg_type_value TEXT NOT NULL,

    PRIMARY KEY (instruction_id)
);

CREATE TABLE instruction_call_module (
    instruction_id UUID NOT NULL REFERENCES instruction(id),
    config UUID NOT NULL REFERENCES config(id),
    method TEXT NOT NULL,
    out_variable_name TEXT NOT NULL,

    PRIMARY KEY (instruction_id)
);

CREATE TABLE instruction_call_module_parameter (
    instruction_call_module_id UUID NOT NULL REFERENCES instruction_call_module(instruction_id),
    sequence SMALLINT NOT NULL,
    name TEXT NOT NULL,
    arg_type SMALLINT NOT NULL,
    arg_type_value TEXT NOT NULL,

    PRIMARY KEY (instruction_call_module_id, sequence)
);
