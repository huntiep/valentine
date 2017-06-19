use Result;
use super::Pool;

pub fn user(pool: &Pool, username: &str) -> Result<()> {
    let conn = pool.get()?;
    let trans = conn.transaction()?;
    trans.execute("DELETE FROM users WHERE username = $1;",
                  &[&username])?;
    trans.execute("DELETE FROM repos WHERE owner = $1;",
                  &[&username])?;
    trans.commit()?;
    Ok(())
}

pub fn repo(pool: &Pool, username: &str, repo_name: &str) -> Result<()> {
    let conn = pool.get()?;
    let trans = conn.transaction()?;
    trans.execute("UPDATE users SET num_repos = num_repos - 1 WHERE username = $1;",
                  &[&username])?;
    trans.execute("DELETE FROM repos WHERE owner = $1 AND name = $1;",
                  &[&username, &repo_name])?;
    trans.commit()?;
    Ok(())
}
