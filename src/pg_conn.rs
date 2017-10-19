// Copied from https://rocket.rs/guide/state/

use std::ops::Deref;

use postgres::Connection;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use r2d2_postgres::PostgresConnectionManager;
use r2d2;

type Pool = r2d2::Pool<PostgresConnectionManager>;

// Connection request guard type: a wrapper around a postgres connection
pub struct PgConn(pub r2d2::PooledConnection<PostgresConnectionManager>);

impl<'a, 'r> FromRequest<'a, 'r> for PgConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<PgConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(PgConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

// For the convenience of using an &DbConn as an &Connection.
impl Deref for PgConn {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
