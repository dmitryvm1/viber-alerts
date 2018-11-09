use diesel::prelude::{PgConnection, QueryResult};
use schema::posts;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
#[derive(Insertable)]
#[table_name="posts"]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(Queryable, Debug)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

impl Post {
    pub fn insert(todo: NewPost, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(posts::table)
            .values(&todo)
            .execute(conn)
    }

    pub fn all(conn: &PgConnection) -> QueryResult<Vec<Post>> {
        posts::dsl::posts.order(posts::id.desc()).load::<Post>(conn)
    }
}
