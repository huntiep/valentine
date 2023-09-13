use Result;
use super::{read, Pool};

pub fn user(pool: &Pool, username: &str) -> Result<()> {
    let owner = read::user_id(pool, username)?;

    let conn = pool.get()?;
    conn.execute(query!("DELETE FROM repos WHERE owner = ?1"), params![owner])?;
    conn.execute(query!("DELETE FROM users WHERE id = ?1"), params![owner])?;
    Ok(())
}

pub fn repo(pool: &Pool, username: &str, repo_name: &str) -> Result<()> {
    let owner = read::user_id(pool, username)?;

    let conn = pool.get()?;
    conn.execute(query!("DELETE FROM repos WHERE owner = ?1 AND name = ?2"), params![owner, repo_name])?;
    conn.execute(query!("UPDATE users SET num_repos = num_repos - 1 WHERE id = ?1"), params![owner])?;
    Ok(())
}

pub fn public_key(pool: &Pool, id: i32) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("DELETE FROM public_keys WHERE id = ?1"), params![id])?;
    Ok(())
}
