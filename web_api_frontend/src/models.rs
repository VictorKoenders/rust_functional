use schema::{
    config, endpoint, instruction, instruction_call_module, instruction_call_module_parameter,
    instruction_json_return,
};
use uuid::Uuid;

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "config"]
#[repr(C)]
pub struct Config {
    pub id: Uuid,
    pub name: String,
    pub path: String,
}

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "endpoint"]
pub struct Endpoint {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub url: String,
}

#[derive(Identifiable, Queryable, Associations, Debug)]
#[table_name = "instruction"]
#[belongs_to(Endpoint, foreign_key = "endpoint")]
pub struct Instruction {
    pub id: Uuid,
    pub endpoint: Uuid,
    pub type_: i16,
    pub sequence: i32,
}

#[derive(Identifiable, Queryable, Associations, Debug)]
#[table_name = "instruction_call_module"]
#[belongs_to(Instruction, foreign_key = "instruction_id")]
#[primary_key(instruction_id)]
pub struct CallModule {
    pub instruction_id: Uuid,
    pub config: Uuid,
    pub method: String,
    pub out_variable_name: String,
}

#[derive(Identifiable, Queryable, Associations, Debug)]
#[table_name = "instruction_call_module_parameter"]
#[belongs_to(CallModule, foreign_key = "instruction_call_module_id")]
#[primary_key(instruction_call_module_id, sequence)]
pub struct CallModuleParameter {
    pub instruction_call_module_id: Uuid,
    pub sequence: i16,
    pub name: String,
    pub arg_type: i16,
    pub arg_type_value: String,
}

#[derive(Identifiable, Queryable, Associations, Debug)]
#[table_name = "instruction_json_return"]
#[belongs_to(Instruction, foreign_key = "instruction_id")]
#[primary_key(instruction_id)]
pub struct JsonReturn {
    pub instruction_id: Uuid,
    pub arg_type: i16,
    pub arg_type_value: String,
}
