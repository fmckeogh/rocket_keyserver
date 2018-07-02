use diesel::pg::PgConnection;
use diesel::prelude::*;
use r2d2;
use r2d2_diesel::ConnectionManager;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use std::env;
use std::ops::Deref;

use schema::keys;

#[derive(Queryable, AsChangeset, Serialize, Deserialize, Insertable)]
pub struct Key {
    pub fingerprint: Vec<u8>,
    pub pgpkey: Vec<u8>,
}

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url());
    Pool::new(manager).expect("db pool")
}

fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn _all(connection: &PgConnection) -> QueryResult<Vec<Key>> {
    keys::table.load::<Key>(&*connection)
}

pub fn get(fingerprint: Vec<u8>, connection: &PgConnection) -> QueryResult<Key> {
    keys::table.find(fingerprint).get_result::<Key>(connection)
}

pub fn insert(key: Key, connection: &PgConnection) -> QueryResult<Key> {
    diesel::insert_into(keys::table)
        .values(&key)
        .get_result(connection)
}

pub fn _update(fingerprint: Vec<u8>, key: Key, connection: &PgConnection) -> QueryResult<Key> {
    diesel::update(keys::table.find(fingerprint))
        .set(&key)
        .get_result(connection)
}

pub fn _delete(fingerprint: Vec<u8>, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(keys::table.find(fingerprint)).execute(connection)
}
