use Result;
use super::{read, repos, users, Pool};

use diesel;
use diesel::prelude::*;

pub fn user(pool: &Pool, username: &str) -> Result<()> {
    let owner = if let Some(id) = read::user_id(pool, username)? {
        id
    } else {
        return Ok(())
    };

    let conn = pool.get()?;
    diesel::delete(users::table.find(owner)).execute(&*conn)?;
    diesel::delete(repos::table.filter(repos::owner.eq(owner))).execute(&*conn)?;
    Ok(())
}

pub fn repo(pool: &Pool, username: &str, repo_name: &str) -> Result<()> {
    let owner = if let Some(id) = read::user_id(pool, username)? {
        id
    } else {
        return Ok(())
    };

    let conn = pool.get()?;
    diesel::delete(repos::table.filter(repos::owner.eq(owner))
                               .filter(repos::name.eq(repo_name)))
        .execute(&*conn)?;
    diesel::update(users::table.find(owner))
        .set(users::num_repos.eq(users::num_repos - 1))
        .execute(&*conn)?;
    Ok(())
}
