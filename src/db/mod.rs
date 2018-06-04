use diesel::{self, r2d2};

pub mod create;
pub mod delete;
pub mod read;
pub mod update;

pub type Pool = r2d2::Pool<r2d2::ConnectionManager<diesel::pg::PgConnection>>;

table! {
    users {
        id -> Integer,
        username -> VarChar,
        email -> VarChar,
        password -> VarChar,
        num_repos -> BigInt,
        is_admin -> Bool,
    }
}

table! {
    public_keys {
        id -> Integer,
        owner -> Integer,
        name -> VarChar,
        fingerprint -> VarChar,
        content -> Text,
    }
}

table! {
    repos {
        id -> BigInt,
        name -> VarChar,
        description -> VarChar,
        owner -> Integer,
        private -> Bool,
        last_updated -> Timestamp,
    }
}
