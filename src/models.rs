use diesel::prelude::{PgConnection, QueryResult};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use schema::users;
#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub viber_id: String,
    pub broadcast: bool,
}

impl User {
    pub fn insert(user: NewUser, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(users::table)
            .values(&user)
            .execute(conn)
    }

    pub fn all(conn: &PgConnection) -> QueryResult<Vec<User>> {
        users::dsl::users.order(users::id.desc()).load::<User>(conn)
    }
}
