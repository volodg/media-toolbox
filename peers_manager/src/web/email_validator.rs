//! Validator executor actor
use actix::prelude::*;

extern crate chrono;
extern crate publicsuffix;
extern crate validator;

use self::chrono::prelude::*;
use self::publicsuffix::List;
use self::validator::*;

pub struct ListWithDate {
    list: List,
    date: DateTime<Utc>,
}

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct ValidateExecutor(pub Option<ListWithDate>);

/// This is only message that this actor can handle
pub struct ValidateEmail {
    pub email: String,
}

impl Message for ValidateEmail {
    type Result = bool;
}

impl Actor for ValidateExecutor {
    type Context = SyncContext<Self>;
}

impl ValidateExecutor {
    fn update_data(&mut self, email: &str) -> bool {
        match List::fetch() {
            Ok(list) => {
                let result = list.parse_email(email).is_ok();
                let list_with_date = ListWithDate {
                    list: list,
                    date: Utc::now(),
                };
                self.0 = Some(list_with_date);
                result
            }
            Err(_) => validate_email(email),
        }
    }
}

impl Handler<ValidateEmail> for ValidateExecutor {
    type Result = bool;

    fn handle(&mut self, msg: ValidateEmail, _: &mut Self::Context) -> bool {
        let one_day = chrono::Duration::days(1);
        let result = self.0.as_ref().map(|el| {
            if Utc::now() - el.date <= one_day {
                Some(el.list.parse_email(&msg.email).is_ok())
            } else {
                None
            }
        });
        match result {
            Some(result) => match result {
                Some(result) => result,
                None => self.update_data(&msg.email)
            },
            None => self.update_data(&msg.email)
        }
    }
}
