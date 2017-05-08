use error::*;
use routes::types::*;
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
