#![feature(test)]

extern crate actix_web;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate itertools;
extern crate r2d2;
extern crate rust_functional;
extern crate uuid;
extern crate web_api_generator;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[cfg(test)]
extern crate test;
#[macro_use]
extern crate enum_primitive;

pub mod endpoint;
pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::{Error, Pool};
use std::env;
use actix_web::{HttpRequest, Responder, App, server, Result, Json};
use actix_web::fs::{NamedFile, StaticFiles};

pub struct AppState {
    pub db: DbConnection,
}

pub struct StateProvider {
    db: DbConnection,
}

impl StateProvider {
    pub fn new() -> Result<StateProvider, Error> {
        let db = establish_connection()?;
        Ok(StateProvider { db })
    }

    pub fn create_state(&self) -> AppState {
        AppState {
            db: self.db.clone(),
        }
    }
}

#[derive(Clone)]
pub struct DbConnection {
    pub(crate) conn: Pool<ConnectionManager<PgConnection>>,
}

pub fn establish_connection() -> Result<DbConnection, Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Ok(DbConnection {
        conn: Pool::new(ConnectionManager::new(database_url))?,
    })
}

fn index(_: HttpRequest<AppState>) -> Result<NamedFile> {
    Ok(NamedFile::open("frontend/index.html")?)
}

fn get_endpoints(req: HttpRequest<AppState>) -> impl Responder {
    let endpoint = endpoint::Endpoints::load(&req.state().db.conn.get().unwrap()).unwrap();
    Json(endpoint)
}

fn main() {
    dotenv::dotenv().expect("Could not load .env file");
    let state_provider = StateProvider::new().unwrap();
    server::new(move || {
        App::with_state(state_provider.create_state())
            .resource("/", |r| r.get().f(index))
            .handler("/dist", StaticFiles::new("frontend/dist"))
            .handler("/node_modules", StaticFiles::new("frontend/node_modules"))
            .resource("/api/endpoints", |r| r.get().f(get_endpoints))
    }).bind("127.0.0.1:8000")
        .unwrap()
        .run();
}
