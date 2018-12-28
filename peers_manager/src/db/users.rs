//! Db executor actor
//extern crate actix;

use actix::prelude::*;
use actix_web::*;
use diesel;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

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

        let conn = &self.0.get().unwrap();

        let result = diesel::insert_into(users)
            .values(&new_user)
            .get_result(conn)
            .map_err(|_| error::ErrorInternalServerError("Error inserting person"))?;

        Ok(result)
    }
}

#[derive(Deserialize)]
pub struct LoginWithEmail {
    pub email: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: Option<i64>,
}

impl Message for LoginWithEmail {
    type Result = Result<LoginResponse, Error>;
}

impl Handler<LoginWithEmail> for DbExecutor {
    type Result = Result<LoginResponse, Error>;

    fn handle(&mut self, msg: LoginWithEmail, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;

        let conn = &self.0.get().unwrap();

        let token = schema::users::table
            .filter(email.eq(msg.email))
            .select(id)
            .first(conn)
            .optional()
            .map_err(|_| error::ErrorInternalServerError("Error login with email"))?;

        let result = LoginResponse { token };

        Ok(result)
    }
}

#[derive(Deserialize)]
pub struct SearchWithKeyword {
    pub keyword: String,
}

impl Message for SearchWithKeyword {
    type Result = Result<Vec<models::User>, Error>;
}

impl Handler<SearchWithKeyword> for DbExecutor {
    type Result = Result<Vec<models::User>, Error>;

    fn handle(&mut self, msg: SearchWithKeyword, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;

        let conn = &self.0.get().unwrap();

        let enquoted_keyword = enquote::enquote('%', &msg.keyword);

        let results = schema::users::table
            .filter(name.like(&enquoted_keyword).or(about.like(&enquoted_keyword)).or(email.eq(&msg.keyword)))
            .get_results::<models::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error user search"))?;

        Ok(results)
    }
}

