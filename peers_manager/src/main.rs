//! Actix web diesel example
//!
//! Diesel does not support tokio, so we have to run it in separate threads.
//! Actix supports sync actors by default, so we going to create sync actor
//! that use diesel. Technically sync actors are worker style actors, multiple
//! of them can run in parallel and process messages from same queue.
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate r2d2;

use actix::prelude::*;
use actix_web::{
    http, middleware, server, App, AsyncResponder, FutureResponse, HttpResponse,
    State, Json,
};

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures::Future;

mod users_db;
mod models;
mod schema;

use users_db::{CreateUser, LoginWithEmail, DbExecutor};

/// State with DbExecutor address
struct AppState {
    db: Addr<DbExecutor>,
}

#[derive(Deserialize)]
struct NewUserInput {
    name: String,
    email: String,
    about: String,
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

fn login_user(
    (login, state): (Json<LoginWithEmail>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    // send async `LoginWithEmail` message to a `DbExecutor`
    state
        .db
        .send(login.into_inner())
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

fn user_search(
    (search, state): (Json<users_db::SearchWithKeyword>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    // send async `SearchWithKeyword` message to a `DbExecutor`
    state
        .db
        .send(search.into_inner())
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
            .middleware(middleware::Logger::default())
            .resource("/users/create_user", |r| r.method(http::Method::POST).with(create_user))
            .resource("/users/login", |r| r.method(http::Method::POST).with(login_user))
            .resource("/users/search", |r| r.method(http::Method::POST).with(user_search))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
