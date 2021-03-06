#![feature(proc_macro_hygiene, decl_macro, trait_alias)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate lazy_static;

mod db;
mod schema;
mod sync_tokens;
mod api;
mod tokens;
mod user;
mod item;
mod lock;

#[cfg(test)]
mod tests;

pub use db::*;

use diesel::prelude::*;
use dotenv::dotenv;
use rocket::Rocket;
use rocket::config::{Config, Environment, Value, Limits};
use std::collections::HashMap;
use std::env;

embed_migrations!();

#[database("db")]
pub struct DbConn(BusyWaitSqliteConnection);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn db_path() -> String {
    env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
}

fn db_config() -> HashMap<&'static str, Value> {
    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();

    database_config.insert("url", Value::from(db_path()));
    databases.insert("db", Value::from(database_config));

    return databases;
}

fn get_environment() -> Environment {
    let v = env::var("SFRS_ENV").unwrap_or("development".to_string());

    if v == "development" {
        Environment::Development
    } else {
        Environment::Production
    }
}

fn build_config() -> Config {
    Config::build(get_environment())
        .extra("databases", db_config())
        .limits(Limits::new().limit("json", 50 * 1024 * 1024))
        .finalize()
        .unwrap()
}

fn run_db_migrations(rocket: Rocket) -> Rocket {
    let db = DbConn::get_one(&rocket).expect("Could not connect to Database");
    match embedded_migrations::run(&*db) {
        Ok(()) => rocket,
        Err(e) => {
            // We should not do anything if database failed to migrate
            panic!("Failed to run database migrations: {:?}", e);
        }
    }
}

pub fn build_rocket() -> Rocket {
    // Make CORS options
    let cors = rocket_cors::CorsOptions {
        allowed_origins: rocket_cors::AllowedOrigins::All,
        allowed_methods: vec![rocket::http::Method::Get, rocket::http::Method::Post]
            .into_iter().map(From::from).collect(),
        allowed_headers: rocket_cors::AllowedHeaders::all(),
        send_wildcard: true,
        ..Default::default()
    }.to_cors().unwrap();

    let r = rocket::custom(build_config())
        .attach(cors)
        .attach(DbConn::fairing())
        .manage(lock::UserLock::new())
        .mount("/", api::routes());
    run_db_migrations(r)
}

fn main() {
    dotenv().ok();
    build_rocket().launch();
}
