use Result;
use types::*;
use super::{Pool, public_keys, repos, users};

use diesel;
use diesel::prelude::*;

pub fn user(pool: &Pool, user: &NewUser) -> Result<()> {
    let conn = pool.get()?;
    diesel::insert_into(users::table).values(user)
        .execute(&*conn)?;
    Ok(())
}

pub fn public_key(pool: &Pool, key: &NewSshKey) -> Result<SshKey> {
    let conn = pool.get()?;
    Ok(diesel::insert_into(public_keys::table).values(key)
        .get_result::<SshKey>(&*conn)?)
}

pub fn repo(pool: &Pool, repo: &Repo) -> Result<()> {
    let conn = pool.get()?;
    diesel::insert_into(repos::table).values(repo).execute(&*conn)?;
    diesel::update(users::table.find(repo.owner))
        .set(users::num_repos.eq(users::num_repos + 1))
        .execute(&*conn)?;
    Ok(())
}
