use error::*;
use routes::types::*;
use super::Pool;

pub fn tables(pool: &Pool) -> Result<()> {
    let conn = pool.get()?;
    conn.batch_execute(include_str!("../sql/create/tables.sql"))?;
    Ok(())
}

pub fn user(pool: &Pool, user: &NewUser) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(include_str!("../sql/create/user.sql"),
                 &[&user.username, &user.password, &user.email])?;
    Ok(())
}

pub fn repo(pool: &Pool, username: &str, repo: &Repo) -> Result<()> {
    let conn = pool.get()?;
    let trans = conn.transaction()?;
    trans.execute("UPDATE users SET num_repos = num_repos + 1 WHERE username = $1;",
                 &[&username])?;
    trans.execute("INSERT INTO repos VALUES ($2, $3, $1);",
                 &[&username, &repo.name, &repo.description])?;
    trans.commit()?;
    Ok(())
}
