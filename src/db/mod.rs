use {diesel, r2d2};
use r2d2_diesel::ConnectionManager;

pub mod create;
//pub mod delete;
pub mod read;

pub type Pool = r2d2::Pool<ConnectionManager<diesel::pg::PgConnection>>;

table! {
    users (uuid) {
        uuid -> Uuid,
        username -> VarChar,
        email -> VarChar,
        password -> VarChar,
        num_repos -> BigInt,
        is_admin -> Bool,
    }
}

table! {
    public_key (id) {
        id -> Integer,
        // TODO
        owner -> Uuid,
        name -> VarChar,
        fingerprint -> VarChar,
        content -> Text,
    }
}

table! {
    repos (uuid) {
        uuid -> Uuid,
        name -> VarChar,
        description -> VarChar,
        // TODO
        owner -> Uuid,
        private -> Bool,
    }
}

table! {
    issues (repo, id) {
        // TODO
        repo -> Uuid,
        id -> BigInt,
        parent -> BigInt,
        subject -> Nullable<VarChar>,
        content -> Text,
        created -> Timestamp,
    }
}
