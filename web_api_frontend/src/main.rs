#![feature(test)]

extern crate actix_web;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate uuid;
extern crate web_api_generator;
extern crate rust_functional;
extern crate itertools;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[cfg(test)]
extern crate test;
#[macro_use] extern crate enum_primitive;

pub mod models;
pub mod schema;
pub mod endpoint;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::BelongingToDsl;
use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    let database_url = "postgres://postgres:postgres@localhost/rust_functional"; //env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));
    let endpoint = endpoint::Endpoints::load(&connection).unwrap();
    println!("{}", serde_json::to_string_pretty(&endpoint).unwrap());
}
/*

    let endpoints: Vec<models::Endpoint> = schema::endpoint::table
        .get_results(&connection)
        .unwrap();
    let instructions =
        models::Instruction::belonging_to(&endpoints)
        .get_results::<models::Instruction>(&connection)
        .unwrap()
        .grouped_by(&endpoints);
    
    let endpoints = endpoints.into_iter().zip(instructions.into_iter()).collect::<Vec<_>>();

    println!("{:?}", endpoints);
    /*
    actix_web::server::new(|| actix_web::App::new())
        .bind("127.0.0.1:8000")
        .unwrap()
        .run();
    */
}
*/
