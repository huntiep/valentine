use Result;
use super::{public_keys, read, repos, users, Pool};

use diesel;
use diesel::prelude::*;

pub fn user(pool: &Pool, username: &str) -> Result<()> {
    let owner = read::user_id(pool, username)?;

    let conn = pool.get()?;
    diesel::delete(users::table.find(owner)).execute(&*conn)?;
    diesel::delete(repos::table.filter(repos::owner.eq(owner))).execute(&*conn)?;
    Ok(())
}

pub fn repo(pool: &Pool, username: &str, repo_name: &str) -> Result<()> {
    let owner = read::user_id(pool, username)?;

    let conn = pool.get()?;
    diesel::delete(repos::table.filter(repos::owner.eq(owner))
                               .filter(repos::name.eq(repo_name)))
        .execute(&*conn)?;
    diesel::update(users::table.find(owner))
        .set(users::num_repos.eq(users::num_repos - 1))
        .execute(&*conn)?;
    Ok(())
}

pub fn public_key(pool: &Pool, id: i32) -> Result<()> {
    let conn = pool.get()?;
    diesel::delete(public_keys::table.find(id)).execute(&*conn)?;
    Ok(())
}
