use {Error, Result};
use types::*;
use super::{Pool, read};

pub fn tables(pool: &Pool) -> Result<()> {
    let conn = pool.get()?;
    conn.batch_execute(include_str!("../sql/create/tables.sql"))?;
    Ok(())
}

pub fn user(pool: &Pool, user: &NewUser) -> Result<()> {
    let conn = pool.get()?;
    let uuid = Uuid::new_v4();
    conn.execute(include_str!("../sql/create/user.sql"),
                 &[&uuid, &user.username, &user.email, &user.password])?;
    Ok(())
}

pub fn repo(pool: &Pool, username: &str, repo: &Repo) -> Result<()> {
    let owner = if let Some(owner) = read::user_uuid(pool, username)? {
        owner
    } else {
        return Err(Error::Postgres("postgres error"));
    };

    let conn = pool.get()?;
    let uuid = Uuid::new_v4();
    let trans = conn.transaction()?;
    trans.execute("UPDATE users SET num_repos = num_repos + 1 WHERE uuid = $1;",
                 &[&owner])?;
    trans.execute("INSERT INTO repos VALUES ($1, $2, $3, $4, $5);",
                 &[&uuid, &repo.name, &repo.description, &owner, &repo.private])?;
    trans.commit()?;
    Ok(())
}
