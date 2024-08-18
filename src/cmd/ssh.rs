use {db, Config, Result};
use git::AccessMode;

use clap::ArgMatches;
use r2d2;
use r2d2_sqlite::SqliteConnectionManager;

use std::{env, process};

pub fn run(config: Config, matches: &ArgMatches) {
    if let Err(e) = _run(config, matches) {
        fail(&format!("Internal error: {}", e), None);
    }
}

fn _run(config: Config, matches: &ArgMatches) -> Result<()> {
    let key_id = matches.get_one::<String>("KEYID").expect("Missing KEYID argument");
    let key_id = key_id[4..].parse::<i32>().expect("Invalid KEYID");
    let cmd = if let Ok(cmd) = env::var("SSH_ORIGINAL_COMMAND") {
        cmd
    } else {
        eprintln!("Hi there, you've successfully authenticated, but Valentine does not provide shell access.");
        eprintln!("If this is unexpected, please log in with password and setup Valentine under another user.");
        return Ok(());
    };

    let (verb, args) = parse_cmd(&cmd);

    let repo_path = args.trim_matches('\'');
    let rr: Vec<&str> = repo_path.splitn(2, '/').collect();
    if rr.len() != 2 {
        fail("Invalid repository path", None);
    }

    let username = rr[0];
    let reponame = rr[1].trim_end_matches(".git");

    let requested_mode = if let Some(mode) = AccessMode::new(&verb) {
        mode
    } else {
        fail("Unknown git command", None);
    };

    // Create db connection pool
    let manager = SqliteConnectionManager::file(config.db_path);
    let pool = r2d2::Pool::new(manager).expect("Failed to create pool");

    if !db::read::user_exists(&pool, username)? {
        fail("Repository owner does not exist", None);
    }

    if !db::read::repo_exists(&pool, username, reponame)? {
        fail("Repository does not exist or you do not have access", None);
    }

    let private = db::read::repo_is_private(&pool, username, reponame)?;

    if requested_mode == AccessMode::Write || private {
        let user = if let Some(user) = db::read::user_by_key_id(&pool, key_id)? {
            user
        } else {
            fail("Internal error", None);
        };

        // TODO: We want to check if a user has *access* rather than if the user
        // owns the repo, whenever we allow access to multiple users
        if !db::read::user_owns_repo(&pool, user, reponame)? {
            fail("Repository does not exist or you do not have access", None);
        };
    }

    let command = process::Command::new(verb)
        .arg(repo_path)
        .current_dir(config.repo_dir)
        .status();
    if let Ok(status) = command {
        if !status.success() {
            fail("internal error 1", None);
        }
    } else {
        fail("internal error 2", None);
    }

    if requested_mode == AccessMode::Write {
        db::update::repo(&pool, username, reponame)?;
    }
    Ok(())
}

fn parse_cmd(cmd: &str) -> (String, String) {
    info!("{}", cmd);
    let cmds: Vec<&str> = cmd.splitn(2, ' ').collect();

    if cmds[1].is_empty() {
        return (String::new(), String::new());
    }

    (cmds[0].to_string(), cmds[1].replacen("'/", "'", 1))
}

fn fail(user_msg: &str, log_msg: Option<&str>) -> ! {
    eprintln!("Valentine: {}", user_msg);
    if let Some(log_msg) = log_msg {
        info!("{}", log_msg);
    }

    process::exit(1);
}
