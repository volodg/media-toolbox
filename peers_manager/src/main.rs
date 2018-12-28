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
extern crate uuid;

//use actix_web::{App, Json, Result, http};

use actix::prelude::*;
use actix_web::{
    http, middleware, server, App, AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Path,
    State, Json, Result, Form,
};

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures::Future;

mod db;
mod models;
mod schema;

use db::{CreateUser, DbExecutor};

/// State with DbExecutor address
struct AppState {
    db: Addr<DbExecutor>,
}

#[derive(Deserialize)]
struct NewUserInput {
    pub name: String,
    pub email: String,
    pub about: String,
}

/// Async request handler
fn create_user(
    (new_user, state): (Json<NewUserInput>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    // send async `CreateUser` message to a `DbExecutor`
    state
        .db
        .send(CreateUser {
            name: new_user.name.clone(),
            email: new_user.email.clone(),
            about: new_user.about.clone(),
        })
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("diesel-example");

    // Start 3 db executor actors
    let database_url = "postgres://postgres:docker@localhost:5432/peers_test";
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));

    // Start http server
    server::new(move || {
        App::with_state(AppState{db: addr.clone()})
            // enable logger
            .middleware(middleware::Logger::default())
            .resource("/create_user", |r| r.method(http::Method::POST).with(create_user))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}

//curl -X GET http://127.0.0.1:8080/test_name
