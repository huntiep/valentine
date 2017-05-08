use bcrypt::{self, DEFAULT_COST};
use hayaku::Request;

pub struct NewUser {
    pub username: String,
    pub password: String,
}

impl NewUser {
    pub fn new(req: &mut Request) -> Option<Self> {
        let username = try_opt!(req.form_value("username"));
        let password = try_opt!(req.form_value("password"));
        let password_confirm = try_opt!(req.form_value("password_confirm"));

        if username.is_empty() || password.is_empty() {
            return None;
        } else if password != password_confirm {
            return None;
        }

        let password_hash = try_opt!(bcrypt::hash(&password, DEFAULT_COST).ok());
        Some(NewUser {
            username: username,
            password: password_hash,
        })
    }
}

pub struct Login {
    pub username: String,
    pub password: String,
}

impl Login {
    pub fn new(req: &mut Request) -> Option<Self> {
        let username = try_opt!(req.form_value("username"));
        let password = try_opt!(req.form_value("password"));

        if username.is_empty() || password.is_empty() {
            return None;
        }

        Some(Login {
            username: username,
            password: password,
        })
    }
}
