use r2d2;
use r2d2_postgres::PostgresConnectionManager;

pub mod create;
pub mod read;

pub type Pool = r2d2::Pool<PostgresConnectionManager>;
