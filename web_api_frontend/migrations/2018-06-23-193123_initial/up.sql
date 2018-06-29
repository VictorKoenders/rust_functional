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
    endpoint UUID NOT NULL REFERENCES endpoint(id) ON DELETE CASCADE,
    type SMALLINT NOT NULL,
    sequence INT NOT NULL,

    UNIQUE(endpoint, sequence)
);

CREATE TABLE instruction_json_return (
    instruction_id UUID NOT NULL REFERENCES instruction(id) ON DELETE CASCADE,
    arg_type SMALLINT NOT NULL,
    arg_type_value TEXT NOT NULL,

    PRIMARY KEY (instruction_id)
);

CREATE TABLE instruction_call_module (
    instruction_id UUID NOT NULL REFERENCES instruction(id) ON DELETE CASCADE,
    config UUID NOT NULL REFERENCES config(id),
    method TEXT NOT NULL,
    out_variable_name TEXT NOT NULL,

    PRIMARY KEY (instruction_id)
);

CREATE TABLE instruction_call_module_parameter (
    instruction_call_module_id UUID NOT NULL REFERENCES instruction_call_module(instruction_id) ON DELETE CASCADE,
    sequence SMALLINT NOT NULL,
    name TEXT NOT NULL,
    arg_type SMALLINT NOT NULL,
    arg_type_value TEXT NOT NULL,

    PRIMARY KEY (instruction_call_module_id, sequence)
);

DO $$
DECLARE config_id UUID;
DECLARE endpoint_id UUID;
BEGIN
    INSERT INTO config (name, path) VALUES ('postgres', 'modules/postgres');
    config_id := (SELECT id FROM config);

    INSERT INTO endpoint (name, description, url) VALUES ('list_users', 'List all users', '/api/users');
    endpoint_id := (SELECT id FROM endpoint);
END $$;