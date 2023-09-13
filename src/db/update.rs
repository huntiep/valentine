use Result;
use super::Pool;

pub fn repo_name(pool: &Pool, username: &str, old_name: &str, new_name: &str) -> Result<()> {
    let repo = super::read::repo_id(pool, username, old_name)?.unwrap();
    let conn = pool.get()?;
    conn.execute(query!("UPDATE repos SET name = ?1 WHERE id = ?2"),
                 params![new_name, repo])?;
    Ok(())
}

pub fn repo(pool: &Pool, username: &str, reponame: &str) -> Result<()> {
    let repo = super::read::repo_id(pool, username, reponame)?.unwrap();
    let conn = pool.get()?;
    let now = ::chrono::Utc::now().naive_utc();
    conn.execute(query!("UPDATE repos SET last_updated = ?1 WHERE id = ?2"),
                 params![now, repo])?;
    Ok(())
}
