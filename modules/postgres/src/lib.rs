#[macro_use]
extern crate lazy_static;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate serde_json;
extern crate uuid;

use serde_json::Value;
use std::collections::HashMap;

lazy_static! {
    static ref POOL: r2d2::Pool<r2d2_postgres::PostgresConnectionManager> = {
        let manager = r2d2_postgres::PostgresConnectionManager::new(
            "postgres://postgres:postgres@localhost/test",
            r2d2_postgres::TlsMode::None,
        ).unwrap();
        r2d2::Pool::new(manager).unwrap()
    };
}

pub fn get_connection() -> r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager> {
    POOL.get().unwrap()
}

pub fn execute_query(
    conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>,
    query: &str,
) -> Vec<HashMap<String, Value>> {
    let query_result = conn.query(query, &[]).unwrap();
    println!("{:?}", query_result.columns());

    let mut result = Vec::with_capacity(query_result.len());
    for row in query_result.into_iter() {
        let mut map = HashMap::with_capacity(row.columns().len());
        for (index, column) in row.columns().iter().enumerate() {
            let value = match column.type_().name() {
                "uuid" => Value::String(row.get::<_, uuid::Uuid>(index).to_string()),
                "text" => Value::String(row.get::<_, String>(index)),
                x => panic!("Unknown type {:?}", x),
            };
            map.insert(column.name().to_owned(), value);
        }
        result.push(map);
    }

    result
}

#[test]
fn test() {
    let connection = get_connection();
    let result = execute_query(&connection, "SELECT * FROM users");
    println!("{}", serde_json::to_string(&result).unwrap());
}
