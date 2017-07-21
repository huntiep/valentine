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
