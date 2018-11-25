use diesel::*;
use schema::users;

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: Option<&'a str>,
    pub viber_id: Option<&'a str>,
    pub broadcast: bool,
}

#[derive(Queryable, Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub email: Option<String>,
    pub viber_id: Option<String>,
    pub broadcast: bool,
}

impl User {
    pub fn insert(user: NewUser, conn: &PgConnection) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(&user)
            .get_result(conn)
    }

    pub fn all(conn: &PgConnection) -> QueryResult<Vec<User>> {
        users::dsl::users.order(users::id.desc()).load::<User>(conn)
    }

    pub fn by_email(user_email: &str, conn: &PgConnection) -> Option<User> {
        let mut results:Vec<User> = users::dsl::users.filter(users::dsl::email.eq(user_email)).load(conn).unwrap();
        results.pop()
    }
}
