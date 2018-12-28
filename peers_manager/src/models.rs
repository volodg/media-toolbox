use super::schema::users;

#[derive(Serialize, Queryable)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub about: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub about: &'a str,
}