use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::QueryResult;
use enum_primitive::FromPrimitive;
use itertools::Itertools;
use models::{
    CallModule as DBCallModule, CallModuleParameter as DBCallModuleParameter, Config as DbConfig,
    Endpoint as DBEndpoint, Instruction as DBInstruction, JsonReturn as DBJsonReturn,
};
use rust_functional::Config as BaseConfig;
use schema;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Endpoints {
    pub configs: Vec<Config>,
    pub endpoints: Vec<Endpoint>,
}

impl Endpoints {
    pub fn load(conn: &PgConnection) -> QueryResult<Endpoints> {
        let configs = schema::config::table
            .get_results::<DbConfig>(&*conn)?
            .into_iter()
            .map(Into::into)
            .collect();
        let endpoints = Endpoint::load(conn)?;
        Ok(Endpoints { configs, endpoints })
    }
}

#[derive(Debug, Serialize)]
pub struct Config {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub config: BaseConfig,
}

impl From<DbConfig> for Config {
    fn from(c: DbConfig) -> Config {
        Config {
            id: c.id,
            name: c.name,
            config: BaseConfig::from_path(&c.path),
            path: c.path,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Endpoint {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub url: String,
    pub instructions: Vec<Instruction>,
}

impl From<DBEndpoint> for Endpoint {
    fn from(endpoint: DBEndpoint) -> Endpoint {
        Endpoint {
            id: endpoint.id,
            name: endpoint.name,
            description: endpoint.description,
            url: endpoint.url,
            instructions: Vec::new(),
        }
    }
}

impl Endpoint {
    pub fn load(conn: &PgConnection) -> QueryResult<Vec<Endpoint>> {
        let endpoints: Vec<DBEndpoint> = schema::endpoint::table.get_results(&*conn).unwrap();
        let instructions = DBInstruction::belonging_to(&endpoints)
            .get_results::<DBInstruction>(&*conn)
            .unwrap();
        let grouped_by_type = instructions
            .into_iter()
            .map(|i| (InstructionType::from_i16(i.type_).unwrap(), i))
            .into_group_map();

        let mut instructions = HashMap::with_capacity(endpoints.len());

        for (key, values) in grouped_by_type {
            for wrapper in Instruction::load(conn, key, values)? {
                instructions
                    .entry(wrapper.endpoint_id)
                    .or_insert_with(|| Vec::new())
                    .push(wrapper);
            }
        }

        let mut endpoints: Vec<Endpoint> = endpoints.into_iter().map(Into::into).collect();

        for (key, value) in instructions {
            let instructions = value
                .into_iter()
                .sorted_by_key(|v| v.sequence)
                .into_iter()
                .map(|v| v.instruction)
                .collect();
            let endpoint = endpoints.iter_mut().find(|e| e.id == key).unwrap();
            endpoint.instructions = instructions;
        }
        Ok(endpoints)
    }
}

struct InstructionWrapper {
    pub endpoint_id: Uuid,
    pub sequence: i32,
    pub instruction: Instruction,
}

impl From<(DBCallModule, Vec<DBCallModuleParameter>)> for Instruction {
    fn from((module, parameters): (DBCallModule, Vec<DBCallModuleParameter>)) -> Instruction {
        Instruction::CallMethod {
            config: module.config,
            method: module.method,
            out_variable_name: module.out_variable_name,
            arguments: parameters
                .into_iter()
                .sorted_by_key(|p| p.sequence)
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<DBJsonReturn> for Instruction {
    fn from(ret: DBJsonReturn) -> Instruction {
        Instruction::JsonReturn {
            arg_type: ArgType::from_i16(ret.arg_type).unwrap(),
            arg_type_value: ret.arg_type_value,
        }
    }
}

enum_from_primitive! {
#[derive(Debug, Hash, PartialEq, Eq)]
enum InstructionType {
    CallMethod = 1,
    JsonReturn = 2,
}
}

#[derive(Debug, Serialize)]
pub enum Instruction {
    CallMethod {
        config: Uuid,
        method: String,
        out_variable_name: String,
        arguments: Vec<CallMethodArgument>,
    },
    JsonReturn {
        arg_type: ArgType,
        arg_type_value: String,
    },
}

impl Instruction {
    fn load(
        conn: &PgConnection,
        key: InstructionType,
        values: Vec<DBInstruction>,
    ) -> QueryResult<Vec<InstructionWrapper>> {
        let ids = values.iter().map(|v| v.id).collect::<Vec<_>>();
        match key {
            InstructionType::CallMethod => {
                let methods = schema::instruction_call_module::table
                    .filter(schema::instruction_call_module::dsl::instruction_id.eq_any(&ids))
                    .get_results::<DBCallModule>(&*conn)?;
                let method_parameters = DBCallModuleParameter::belonging_to(&methods)
                    .get_results::<DBCallModuleParameter>(&*conn)?
                    .grouped_by(&methods);
                // let methods: Vec<Instruction> = methods.into_iter().zip(method_parameters.into_iter()).map(Into::into).collect();
                let mut result = Vec::with_capacity(methods.len());
                for (method, parameters) in methods.into_iter().zip(method_parameters.into_iter()) {
                    let instruction = values
                        .iter()
                        .find(|v| v.id == method.instruction_id)
                        .unwrap();
                    result.push(InstructionWrapper {
                        instruction: (method, parameters).into(),
                        sequence: instruction.sequence,
                        endpoint_id: instruction.endpoint,
                    })
                }
                Ok(result)
            }
            InstructionType::JsonReturn => {
                let instructions = schema::instruction_json_return::table
                    .filter(schema::instruction_json_return::dsl::instruction_id.eq_any(&ids))
                    .get_results::<DBJsonReturn>(&*conn)?;
                let mut result = Vec::with_capacity(instructions.len());
                for instruction in instructions {
                    let definition = values.iter().find(|v| v.id == instruction.instruction_id).unwrap();
                    result.push(InstructionWrapper {
                        instruction: instruction.into(),
                        sequence: definition.sequence,
                        endpoint_id: definition.endpoint
                    });
                }
                Ok(result)
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CallMethodArgument {
    pub name: String,
    pub arg_type: ArgType,
    pub arg_type_value: String,
}

impl From<DBCallModuleParameter> for CallMethodArgument {
    fn from(param: DBCallModuleParameter) -> CallMethodArgument {
        CallMethodArgument {
            name: param.name,
            arg_type: ArgType::from_i16(param.arg_type).unwrap(),
            arg_type_value: param.arg_type_value,
        }
    }
}

enum_from_primitive! {
#[derive(Debug, Hash, PartialEq, Eq, Serialize)]
pub enum ArgType {
    Parameter = 1,
    String = 2,
}
}
