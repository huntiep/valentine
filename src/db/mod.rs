pub mod create;
pub mod delete;
pub mod read;
pub mod update;

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
