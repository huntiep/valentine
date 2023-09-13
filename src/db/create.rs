use Result;
use types::*;
use super::Pool;

pub fn user(pool: &Pool, user: &NewUser) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("INSERT INTO users (username, password, email, num_repos) VALUES (?1, ?2, ?3, ?4)"),
                 params![user.username, user.password, user.email, user.num_repos])?;
    Ok(())
}

pub fn public_key(pool: &Pool, key: &NewSshKey) -> Result<SshKey> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("INSERT INTO public_keys (owner, name, fingerprint, content) VALUES (?1, ?2, ?3, ?4) RETURNING id"))?;
    let id: i32 = stmt.query_row(params![key.owner, key.name, key.fingerprint, key.content], |row| row.get(0))?;
    Ok(SshKey {
        id: id,
        owner: key.owner,
        name: key.name,
        fingerprint: key.fingerprint,
        content: key.content,
    })
}

pub fn repo(pool: &Pool, repo: &Repo) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(query!("INSERT INTO repos (name, description, owner, private) VALUES (?1, ?2, ?3, ?4)"),
                 params![repo.name, repo.description, repo.owner, repo.private])?;
    conn.execute(query!("UPDATE users SET num_repos = num_repos + 1 WHERE id = ?1"), params![repo.owner])?;
    Ok(())
}
