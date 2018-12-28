//! Db executor actor
//extern crate actix;

use actix::prelude::*;
use actix_web::*;
use diesel;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid;

use models;
use schema;

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

/// This is only message that this actor can handle, but it is easy to extend
/// number of messages.
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub about: String,
}

impl Message for CreateUser {
    type Result = Result<models::User, Error>;
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<CreateUser> for DbExecutor {
    type Result = Result<models::User, Error>;

    fn handle(&mut self, msg: CreateUser, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;

        let new_user = models::NewUser {
            name: &msg.name,
            email: &msg.email,
            about: &msg.about,
        };

        let conn: &PgConnection = &self.0.get().unwrap();

        let result = diesel::insert_into(users)
            .values(&new_user)
            .get_result::<models::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error inserting person"))?;

        Ok(result)
    }
}