use actix::Handler;
use workers::WebWorker;
use models::User;
use actix::Message;
use workers::CustomError;

pub struct UserByEmail(String);

impl Message for UserByEmail {
    type Result = Result<User, failure::Error>;
}

impl Handler<UserByEmail> for WebWorker {
    type Result = Result<User, failure::Error>;

    fn handle(&mut self, msg: UserByEmail, _: &mut Self::Context) -> Self::Result {
        let conn = self.app_state.pool.get().unwrap();
        User::by_email(msg.0.as_str(), &conn).ok_or(failure::Error::from(CustomError{ msg:"no such user".to_owned() }))
    }
}
