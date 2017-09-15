use {Context, Result};

use std::process;
use std::io::Write;

pub fn info(ctx: &Context, username: &str, repo_name: &str) -> Result<Vec<u8>> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(username);
    root_dir.push(repo_name);

    let command = process::Command::new("git-upload-pack")
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(root_dir)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .output()?;

    Ok(command.stdout)
}

pub fn pull(ctx: &Context, username: &str, repo_name: &str, body: &[u8]) -> Result<Vec<u8>> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(username);
    root_dir.push(repo_name);

    let mut command = process::Command::new("git-upload-pack")
        .arg("--stateless-rpc")
        .arg(root_dir)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()?;

    {
        let stdin = command.stdin.as_mut().unwrap();
        stdin.write_all(body)?;
    }
    let output = command.wait_with_output()?;

    if !output.status.success() {
        // TODO
    }

    Ok(output.stdout)
}
