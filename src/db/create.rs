use {Error, Result};
use types::*;
use super::{Pool, public_keys, repos, users};

use diesel;
use diesel::prelude::*;

/*pub fn tables(pool: &Pool) -> Result<()> {
    let conn = pool.get()?;
    conn.batch_execute(include_str!("../sql/create/tables.sql"))?;
    Ok(())
}*/

pub fn user(pool: &Pool, user: &NewUser) -> Result<()> {
    let conn = pool.get()?;
    diesel::insert(user).into(users::table)
        .execute(&*conn)?;
    Ok(())
}

pub fn public_key(pool: &Pool, key: &NewSshKey) -> Result<i32> {
    let conn = pool.get()?;
    let key = diesel::insert(key).into(public_keys::table)
        .get_result::<SshKey>(&*conn)?;
    Ok(key.id)
}

pub fn repo(pool: &Pool, repo: &Repo) -> Result<()> {
    let conn = pool.get()?;
    diesel::insert(repo).into(repos::table).execute(&*conn)?;
    diesel::update(users::table.find(repo.owner))
        .set(users::num_repos.eq(users::num_repos + 1))
        .execute(&*conn)?;
    Ok(())
}
