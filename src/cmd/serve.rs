use Config;
use git::AccessMode;

use toml;

use std::{env, fs, process};
use std::io::{Read, Write};

pub fn run(config: Config) {
    let cmd = if let Ok(cmd) = env::var("SSH_ORIGINAL_COMMAND") {
        cmd
    } else {
        println!("Hi there, you've successfully authenticated, but Valentine does not provide shell access.");
        println!("If this is unexpected, please log in with password and setup Valentine under another user.");
        return;
    };

    let (verb, args) = parse_cmd(&cmd);

    let repo_path = args.trim_matches('\'');
    let rr: Vec<&str> = repo_path.splitn(2, '/').collect();
    if rr[1].is_empty() {
        fail("Invalid repository path", None);
    }

    let username = rr[0];
    let reponame = rr[1].trim_right_matches(".git");

    let requested_mode = if let Some(mode) = AccessMode::new(&verb) {
        mode
    } else {
        fail("Unknown git command", None);
    };

    if requested_mode == AccessMode::Write {
    } else {
    }

    eprintln!("{} {}", verb, repo_path);
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
