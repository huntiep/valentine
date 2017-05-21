use error::*;
use routes::types::*;
use templates::*;
use super::Pool;

pub fn check_login(pool: &Pool, login: &Login) -> Result<bool> {
    let conn = pool.get()?;
    let rows = conn.query(include_str!("../sql/read/check_login.sql"),
                          &[&login.username])?;
    if rows.is_empty() {
        Ok(false)
    } else {
        let row = rows.get(0);
        let password_hash: String = row.get(1);
        let valid = ::bcrypt::verify(&login.password, &password_hash)?;
        Ok(valid)
    }
}

pub fn user_exists(pool: &Pool, username: &str) -> Result<bool> {
    let conn = pool.get()?;
    let rows = conn.query(include_str!("../sql/read/user_exists.sql"),
                          &[&username])?;
    Ok(!rows.is_empty())
}

pub fn repo_exists(pool: &Pool, username: &str, reponame: &str) -> Result<bool> {
    let conn = pool.get()?;
    let rows = conn.query(include_str!("../sql/read/repo_exists.sql"),
                          &[&username, &reponame])?;
    Ok(!rows.is_empty())
}

pub fn user(pool: &Pool, username: &str) -> Result<Option<User>> {
    if !user_exists(pool, username)? {
        info!("user doesn't exist!");
        return Ok(None);
    }

    let conn = pool.get()?;
    let rows = conn.query(include_str!("../sql/read/user.sql"),
                          &[&username])?;

    let mut repos = Vec::new();
    for row in rows.iter() {
        let repo = Repo {
            name: row.get(0),
            description: row.get(1),
        };
        repos.push(repo);
    }

    Ok(Some(User {
        username: username.to_string(),
        repos: repos,
    }))
}
