use actix::Handler;
use workers::WebWorker;
use models::User;
use actix::Message;
use workers::CustomError;
use models::NewUser;

pub struct UserByEmail(pub String);

impl Message for UserByEmail {
    type Result = Result<User, failure::Error>;
}

impl Handler<UserByEmail> for WebWorker {
    type Result = Result<User, failure::Error>;

    fn handle(&mut self, msg: UserByEmail, _: &mut Self::Context) -> Self::Result {
        let conn = self.app_state.pool.get().unwrap();
        User::by_email(msg.0.as_str(), &conn)
            .ok_or(failure::Error::from(CustomError{ msg:"no such user".to_owned() }))
    }
}

pub struct RegisterUser(pub String);

impl Message for RegisterUser {
    type Result = Result<User, failure::Error>;
}

impl Handler<RegisterUser> for WebWorker {
    type Result = Result<User, failure::Error>;

    fn handle(&mut self, msg: RegisterUser, ctx: &mut Self::Context) -> <Self as Handler<RegisterUser>>::Result {
        let conn = self.app_state.pool.get().unwrap();
        let res = User::insert(NewUser {
            email: Some(msg.0.as_str()),
            broadcast: false,
            viber_id: None
        }, &conn);
        res.map_err(|e|{
            failure::Error::from(e)
        })
    }
}