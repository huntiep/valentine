use Result;
use super::{Pool, repos};

use diesel;
use diesel::prelude::*;

pub fn repo_name(pool: &Pool, username: &str, old_name: &str, new_name: &str) -> Result<()> {
    let repo = super::read::repo_id(pool, username, old_name)?.unwrap();
    let conn = pool.get()?;
    diesel::update(repos::table.find(repo))
        .set(repos::name.eq(new_name))
        .execute(&*conn)?;
    Ok(())
}

pub fn repo(pool: &Pool, username: &str, reponame: &str) -> Result<()> {
    let repo = super::read::repo_id(pool, username, reponame)?.unwrap();
    let conn = pool.get()?;
    let now = ::chrono::Utc::now().naive_utc();
    diesel::update(repos::table.find(repo))
        .set(repos::last_updated.eq(now))
        .execute(&*conn)?;
    Ok(())
}
