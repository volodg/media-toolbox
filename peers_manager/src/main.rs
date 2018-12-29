//! Actix web diesel example
//!
//! Diesel does not support tokio, so we have to run it in separate threads.
//! Actix supports sync actors by default, so we going to create sync actor
//! that use diesel. Technically sync actors are worker style actors, multiple
//! of them can run in parallel and process messages from same queue.
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate r2d2;

use actix::prelude::*;
use actix_web::{http, middleware, server, App};

use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;

mod db;
mod models;
mod schema;
mod web;

use db::users::DbExecutor;
use web::app::AppState;
use web::users::create::create_user;
use web::users::handlers::{login_user, user_search};

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("diesel-example");

    // Start 3 db executor actors
    let database_url = "postgres://postgres:docker@localhost:5432/peers_dev";
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let addr1 = SyncArbiter::start(3, move || DbExecutor(pool.clone()));

    let addr2 = SyncArbiter::start(3, || web::email_validator::ValidateExecutor(None));

    // Start http server
    server::new(move || {
        App::with_state(AppState {
            db: addr1.clone(),
            email_validator: addr2.clone(),
        })
        .middleware(middleware::Logger::default())
        .resource("/users/create_user", |r| {
            r.method(http::Method::POST).with(create_user)
        })
        .resource("/users/login", |r| {
            r.method(http::Method::POST).with(login_user)
        })
        .resource("/users/search", |r| {
            r.method(http::Method::POST).with(user_search)
        })
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
