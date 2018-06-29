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
use web_api_generator;
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
            for wrapper in Instruction::load(conn, key, &values)? {
                instructions
                    .entry(wrapper.endpoint_id)
                    .or_insert_with(Vec::new)
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

    pub fn load_one(conn: &PgConnection, id: Uuid) -> QueryResult<Option<Endpoint>> {
        let endpoint: Option<DBEndpoint> = schema::endpoint::table
            .filter(schema::endpoint::id.eq(id))
            .get_result(&*conn)
            .optional()
            .unwrap();
        let endpoint = match endpoint {
            Some(e) => e,
            None => return Ok(None),
        };
        let instructions = DBInstruction::belonging_to(&endpoint)
            .get_results::<DBInstruction>(&*conn)
            .unwrap();
        let grouped_by_type = instructions
            .into_iter()
            .map(|i| (InstructionType::from_i16(i.type_).unwrap(), i))
            .into_group_map();

        let mut instructions = Vec::new();

        for (key, values) in grouped_by_type {
            for wrapper in Instruction::load(conn, key, &values)? {
                instructions.push(wrapper);
            }
        }

        let mut endpoint: Endpoint = endpoint.into();

        let instructions = instructions
            .into_iter()
            .sorted_by_key(|v| v.sequence)
            .into_iter()
            .map(|v| v.instruction)
            .collect();
        endpoint.instructions = instructions;
        Ok(Some(endpoint))
    }

    pub fn generate(&self, conn: &PgConnection) -> QueryResult<HashMap<String, String>> {
        let mut builder = web_api_generator::Builder::default();
        let config_ids = self
            .instructions
            .iter()
            .filter_map(|i| i.get_config_id())
            .unique()
            .collect::<Vec<_>>();
        let result = {
            let configs: Vec<Config> = schema::config::table
                .filter(schema::config::id.eq_any(&config_ids))
                .get_results::<DbConfig>(conn)?
                .into_iter()
                .map(Into::into)
                .collect();
            let result = {
                let configs = configs.into_iter().map(|c| (c.id, Rc::new(c.config))).collect::<Vec<_>>();
                for config in &configs {
                    builder.add_module(config.1.clone());
                }
                builder.add_endpoint(web_api_generator::EndPoint {
                    name: self.name.clone(),
                    url: self.url.clone(),
                    instructions: self
                        .instructions
                        .iter()
                        .map(|i| (i, &configs))
                        .map(Convert::from)
                        .collect(),
                });
                let result = builder.build();
                result
            };
            result
        };
        Ok(result)
    }

    pub fn insert_or_update(&mut self, conn: &PgConnection) -> QueryResult<()> {
        let db = DBEndpoint {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            url: self.url.clone(),
        };

        let id: Uuid = ::diesel::insert_into(schema::endpoint::table)
            .values(&db)
            .on_conflict(schema::endpoint::dsl::id)
            .do_update()
            .set(&db)
            .returning(schema::endpoint::dsl::id)
            .get_result(conn)?;

        self.id = id;

        ::diesel::sql_query("UPDATE instruction SET sequence = -1 - sequence WHERE endpoint = $1")
            .bind::<::diesel::sql_types::Uuid, _>(id)
            .execute(conn)?;

        for (index, instruction) in self.instructions.iter_mut().enumerate() {
            instruction.insert_or_update(id, index as i32, conn)?;
        }

        ::diesel::delete(
            schema::instruction::table.filter(schema::instruction::dsl::sequence.lt(0)),
        ).execute(conn)?;
        Ok(())
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
            id: module.instruction_id,
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
            id: ret.instruction_id,
            arg_type: ArgType::from_i16(ret.arg_type).unwrap(),
            arg_type_value: ret.arg_type_value,
        }
    }
}

enum_from_primitive! {
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum InstructionType {
    CallMethod = 1,
    JsonReturn = 2,
}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Instruction {
    CallMethod {
        id: Uuid,
        config: Uuid,
        method: String,
        out_variable_name: String,
        arguments: Vec<CallMethodArgument>,
    },
    JsonReturn {
        id: Uuid,
        arg_type: ArgType,
        arg_type_value: String,
    },
}

impl Instruction {
    fn load(
        conn: &PgConnection,
        key: InstructionType,
        values: &[DBInstruction],
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
                    let definition = values
                        .iter()
                        .find(|v| v.id == instruction.instruction_id)
                        .unwrap();
                    result.push(InstructionWrapper {
                        instruction: instruction.into(),
                        sequence: definition.sequence,
                        endpoint_id: definition.endpoint,
                    });
                }
                Ok(result)
            }
        }
    }

    fn get_type(&self) -> i16 {
        match self {
            Instruction::CallMethod { .. } => InstructionType::CallMethod as i16,
            Instruction::JsonReturn { .. } => InstructionType::JsonReturn as i16,
        }
    }

    fn get_id(&self) -> Uuid {
        match self {
            Instruction::CallMethod { id, .. } | Instruction::JsonReturn { id, .. } => *id,
        }
    }
    fn get_config_id(&self) -> Option<Uuid> {
        match self {
            Instruction::CallMethod { config, .. } => Some(*config),
            _ => None,
        }
    }
    fn set_id(&mut self, new_id: Uuid) {
        match self {
            Instruction::CallMethod { id, .. } | Instruction::JsonReturn { id, .. } => *id = new_id,
        }
    }
    fn insert_or_update(
        &mut self,
        endpoint: Uuid,
        sequence: i32,
        conn: &PgConnection,
    ) -> QueryResult<()> {
        let mut id = self.get_id();
        {
            let type_ = self.get_type();
            let instruction = DBInstruction {
                id,
                type_,
                endpoint,
                sequence,
            };
            id = ::diesel::insert_into(schema::instruction::table)
                .values(&instruction)
                .on_conflict(schema::instruction::dsl::id)
                .do_update()
                .set(&instruction)
                .returning(schema::instruction::dsl::id)
                .get_result(conn)?;
            self.set_id(id);
        }
        match self {
            Instruction::CallMethod {
                id,
                config,
                method,
                out_variable_name,
                arguments,
            } => {
                let module = DBCallModule {
                    instruction_id: *id,
                    config: *config,
                    method: method.clone(),
                    out_variable_name: out_variable_name.clone(),
                };
                ::diesel::delete(
                    schema::instruction_json_return::table.filter(
                        schema::instruction_json_return::dsl::instruction_id.eq(id.clone()),
                    ),
                ).execute(conn)?;
                ::diesel::insert_into(schema::instruction_call_module::table)
                    .values(&module)
                    .on_conflict(schema::instruction_call_module::dsl::instruction_id)
                    .do_update()
                    .set(&module)
                    .execute(conn)?;
                for (index, arg) in arguments.iter().enumerate() {
                    let param = DBCallModuleParameter {
                        instruction_call_module_id: *id,
                        sequence: index as i16,
                        name: arg.name.clone(),
                        arg_type: arg.arg_type as i16,
                        arg_type_value: arg.arg_type_value.clone(),
                    };
                    ::diesel::insert_into(schema::instruction_call_module_parameter::table)
                        .values(&param)
                        .on_conflict((schema::instruction_call_module_parameter::dsl::instruction_call_module_id, schema::instruction_call_module_parameter::dsl::sequence))
                        .do_update()
                        .set(&param)
                        .execute(conn)?;
                }
                ::diesel::delete(
                    schema::instruction_call_module_parameter::table.filter(
                        schema::instruction_call_module_parameter::dsl::instruction_call_module_id
                            .eq(id.clone())
                            .and(
                                schema::instruction_call_module_parameter::dsl::sequence
                                    .ge(arguments.len() as i16),
                            ),
                    ),
                ).execute(conn)?;
            }
            Instruction::JsonReturn {
                id,
                arg_type,
                arg_type_value,
            } => {
                let db = DBJsonReturn {
                    instruction_id: *id,
                    arg_type: *arg_type as i16,
                    arg_type_value: arg_type_value.clone(),
                };
                ::diesel::delete(
                    schema::instruction_call_module_parameter::table.filter(
                        schema::instruction_call_module_parameter::dsl::instruction_call_module_id
                            .eq(id.clone()),
                    ),
                ).execute(conn)?;
                ::diesel::delete(
                    schema::instruction_call_module::table.filter(
                        schema::instruction_call_module::dsl::instruction_id.eq(id.clone()),
                    ),
                ).execute(conn)?;
                ::diesel::insert_into(schema::instruction_json_return::table)
                    .values(&db)
                    .on_conflict(schema::instruction_json_return::dsl::instruction_id)
                    .do_update()
                    .set(&db)
                    .execute(conn)?;
            }
        }
        Ok(())
    }
}

trait Convert<T> {
    fn from(t: T) -> Self;
}

impl<'a> Convert<(&'a Instruction, &'a Vec<(Uuid, Rc<web_api_generator::Config>)>)> for web_api_generator::Instruction {
    fn from(
        (instruction, configs): (&'a Instruction, &'a Vec<(Uuid, Rc<web_api_generator::Config>)>),
    ) -> web_api_generator::Instruction {
        match instruction {
            Instruction::JsonReturn {
                arg_type,
                arg_type_value,
                ..
            } => web_api_generator::Instruction::Json(match arg_type {
                ArgType::String => {
                    web_api_generator::InstructionParameter::String(arg_type_value.clone())
                }
                ArgType::Parameter => {
                    web_api_generator::InstructionParameter::Variable(arg_type_value.clone())
                }
            }),
            Instruction::CallMethod {
                config,
                method,
                out_variable_name,
                arguments,
                ..
            } => web_api_generator::Instruction::BaseInstruction(
                web_api_generator::BaseInstruction::CallModule {
                    config: configs.iter().find(|c| &c.0 == config).unwrap().1.clone(),
                    method: method.clone(),
                    out_variable_name: out_variable_name.clone(),
                    parameters: arguments.iter().map(Into::into).collect(),
                },
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallMethodArgument {
    pub name: String,
    pub arg_type: ArgType,
    pub arg_type_value: String,
}

impl<'a> From<&'a CallMethodArgument> for (String, web_api_generator::InstructionParameter) {
    fn from(m: &'a CallMethodArgument) -> (String, web_api_generator::InstructionParameter) {
        (
            m.name.clone(),
            match m.arg_type {
                ArgType::String => {
                    web_api_generator::InstructionParameter::String(m.arg_type_value.clone())
                }
                ArgType::Parameter => {
                    web_api_generator::InstructionParameter::Variable(m.arg_type_value.clone())
                }
            },
        )
    }
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
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum ArgType {
    Parameter = 1,
    String = 2,
}
}
