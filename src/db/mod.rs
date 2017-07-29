use {diesel, r2d2};
use r2d2_diesel::ConnectionManager;

pub mod create;
pub mod delete;
pub mod read;
pub mod update;

pub type Pool = r2d2::Pool<ConnectionManager<diesel::pg::PgConnection>>;

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

numeric_expr!(users::num_repos);

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
        issue_id -> BigInt,
    }
}

numeric_expr!(repos::issue_id);

table! {
    issues (repo, id) {
        repo -> BigInt,
        id -> BigInt,
        parent -> BigInt,
        name -> Nullable<VarChar>,
        subject -> Nullable<VarChar>,
        content -> Text,
        created -> Timestamp,
        thread -> Bool,
    }
}
