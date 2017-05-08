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
    info!("got pool");
    conn.execute(include_str!("../sql/create/user.sql"),
                 &[&user.username, &user.password])?;
    info!("executed query");
    Ok(())
}
