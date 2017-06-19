use std::{env, fs, process};
use std::io::Write;

pub fn run() {
    let mut log = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("/home/git/valentine/val.log")
        .expect("failed to open log file");

    let cmd = if let Ok(cmd) = env::var("SSH_ORIGINAL_COMMAND") {
        cmd
    } else {
        println!("Hi there, you've successfully authenticated, but Valentine does not provide shell access.");
        println!("If this is unexpected, please log in with password and setup Valentine under another user.");
        writeln!(log, "Hi there, you've successfully authenticated, but Valentine does not provide shell access.");
        writeln!(log, "If this is unexpected, please log in with password and setup Valentine under another user.");
        return;
    };

    writeln!(log, "{}", cmd);
    let (verb, args) = parse_cmd(&cmd);

    let repo_path = args.trim_matches('\'');
    let rr: Vec<&str> = repo_path.splitn(2, '/').collect();
    if rr[1].is_empty() {
        fail("Invalid repository path", None);
    }

    let username = rr[0];
    let reponame = rr[1].trim_right_matches(".git");

    let requested_mode = match verb.as_str() {
        "git-upload-pack" => AccessMode::Read,
        "git-upload-archive" => AccessMode::Read,
        "git-receive-pack" => AccessMode::Write,
        _ => fail("Unknown git command", None),
    };

    writeln!(log, "requested mode: {:?}", requested_mode);

    if requested_mode == AccessMode::Write {
    } else {
    }

    let mut current_dir = env::current_dir().unwrap();
    current_dir.push("val-repos");
//    env::set_current_dir(current_dir).unwrap();
    eprintln!("{} {}", verb, repo_path);
    let command = process::Command::new(verb)
        .arg(repo_path)
        .current_dir(current_dir)
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
    }

    process::exit(1);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AccessMode {
    Read,
    Write,
}
