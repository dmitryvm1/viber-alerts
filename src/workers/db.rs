use actix::Handler;
use workers::WebWorker;
use actix_web::Error;
use models::User;
use actix_web::error;
/*
#[derive(Message)]
pub enum DbCommand {
    UserByEmail(String),
    AllUsers
}

impl Handler<DbCommand> for WebWorker {
    type Result = Result<Vec<User>, Error>;

    fn handle(&mut self, cmd: DbCommand, _: &mut Self::Context) -> Self::Result {
        match cmd {
            DbCommand::UserByEmail(email) => {
                User::by_email
            }
        }
        Users::all(self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error inserting task"))
    }
}
*/