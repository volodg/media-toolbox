//! Validator executor actor
use actix::prelude::*;

extern crate chrono;
pub extern crate publicsuffix;

use self::chrono::prelude::*;
use self::publicsuffix::List;

//#[derive(Copy)]
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
    type Result = Result<bool, publicsuffix::Error>;
}

impl Actor for ValidateExecutor {
    type Context = SyncContext<Self>;
}

impl ValidateExecutor {
    fn update_data(&mut self, email: &str) -> Result<bool, publicsuffix::Error> {
        let list = List::fetch()?;

        let result = list.parse_email(email).is_ok();

        let list_with_date = ListWithDate {
            list: list,
            date: Utc::now(),
        };
        self.0 = Some(list_with_date);

        Ok(result)
    }
}

impl Handler<ValidateEmail> for ValidateExecutor {
    type Result = Result<bool, publicsuffix::Error>;

    fn handle(&mut self, msg: ValidateEmail, _: &mut Self::Context) -> Self::Result {
        let date: Option<DateTime<Utc>> = self.0.as_ref().map(|el| el.date);

        let one_day = chrono::Duration::days(1);

        match date {
            Some(ref date) => {
                if Utc::now() - *date <= one_day {
                    return Ok(self
                        .0
                        .as_ref()
                        .unwrap()
                        .list
                        .parse_email(&msg.email)
                        .is_ok());
                }
            }
            None => {}
        }

        self.update_data(&msg.email)
    }
}
