//! Db executor actor
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

#[derive(Debug)]
pub enum CreateUserError {
    UserAlreadyExists,
    DbError(diesel::result::Error),
}

impl Message for CreateUser {
    type Result = Result<models::User, CreateUserError>;
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<CreateUser> for DbExecutor {
    type Result = Result<models::User, CreateUserError>;

    fn handle(&mut self, msg: CreateUser, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;

        let new_user = models::NewUser {
            name: &msg.name,
            email: &msg.email,
            about: &msg.about,
        };

        let conn = &self.0.get().unwrap();

        diesel::insert_into(users)
            .values(&new_user)
            .get_result::<models::User>(conn)
            .map_err(|db_error| {
                match &db_error {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        db_error,
                    ) => match db_error.constraint_name() {
                        Some("email") => return CreateUserError::UserAlreadyExists,
                        _ => {}
                    },
                    _ => {}
                };

                CreateUserError::DbError(db_error)
            })
    }
}

#[derive(Deserialize, Serialize)]
pub struct LoginWithEmail {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: i64,
}

#[derive(Debug)]
pub enum LoginError {
    InvalidCredentials,
    DbError(diesel::result::Error),
}

impl Message for LoginWithEmail {
    type Result = Result<LoginResponse, LoginError>;
}

impl Handler<LoginWithEmail> for DbExecutor {
    type Result = Result<LoginResponse, LoginError>;

    fn handle(&mut self, msg: LoginWithEmail, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;

        let conn = &self.0.get().unwrap();

        let token = schema::users::table
            .filter(email.eq(msg.email))
            .select(id)
            .first(conn)
            .optional()
            .map_err(|db_error| LoginError::DbError(db_error))?;

        match token {
            Some(token) => Ok(LoginResponse { token }),
            None => Err(LoginError::InvalidCredentials),
        }
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
            .filter(
                name.like(&enquoted_keyword)
                    .or(about.like(&enquoted_keyword))
                    .or(email.eq(&msg.keyword)),
            )
            .get_results::<models::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error user search"))?;

        Ok(results)
    }
}
