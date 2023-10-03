#[macro_use] extern crate bart_derive;
extern crate base64;
extern crate bcrypt;
#[macro_use] extern crate check_psql;
extern crate chrono;
#[macro_use] extern crate clap;
extern crate dirs;
extern crate env_logger;
#[macro_use] extern crate explode;
extern crate git2;
#[macro_use] extern crate hayaku;
extern crate html_escape;
extern crate humansize;
#[macro_use] extern crate log;
extern crate pulldown_cmark;
#[macro_use] extern crate quick_error;
extern crate rand;
extern crate r2d2;
extern crate r2d2_sqlite;
#[macro_use] extern crate rusqlite;
extern crate rusqlite_migration;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate sessions;
extern crate sha2;
extern crate time;
extern crate toml;

#[macro_use] mod macros;
mod cmd;
mod db;
mod git;
mod routes;
mod templates;
mod types;

use clap::{Command, Arg};

use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type Result<T> = ::std::result::Result<T, Error>;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Base64(err: ::base64::DecodeError) {
            from()
        }
        Bcrypt(err: ::bcrypt::BcryptError) {
            from()
        }
        Chrono(err: ::chrono::ParseError) {
            from()
        }
        Git(err: ::git2::Error) {
            from()
        }
        Io(err: ::std::io::Error) {
            from()
        }
        R2D2(err: ::r2d2::Error) {
            from()
        }
        Session(err: sessions::Error) {
            from()
        }
        Sqlite(err: rusqlite::Error) {
            from()
        }
    }
}

pub struct Context {
    pub db_pool: db::Pool,
    pub mount: String,
    pub logins: Arc<Mutex<sessions::SessionSet>>,
    pub name: String,
    pub url: String,
    pub ssh: String,
    pub signup: bool,
    pub repo_dir: PathBuf,
    pub ssh_dir: PathBuf,
    pub bin_path: PathBuf,
    pub config_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub repo_dir: PathBuf,
    pub ssh_dir: Option<PathBuf>,
    pub sessions_dir: PathBuf,
    pub db_path: PathBuf,
    pub mount: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub ssh: String,
    pub signup: Option<bool>,
    pub addr: Option<SocketAddr>,
}

fn main() {
    env_logger::init();

    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::new("config")
             .short('c')
             .long("config")
             .value_name("FILE")
             .help("Specifies where to find the config file"))
        .subcommand(Command::new("backup")
                    .about("Create a backup of the database and user repositories")
                    .arg(Arg::new("FILE")
                         .help("The file to output the backup to e.g. val.tgz")
                         .required(true)
                         .index(1)))
        .subcommand(Command::new("ssh")
                    .about("Command used for ssh. Not intended to be used directly")
                    .arg(Arg::new("KEYID")
                         .help("The id of this ssh key")
                         .required(true)
                         .index(1)))
        .subcommand(Command::new("web")
                    .about("Run the valentine server"))
        .get_matches();

    // Read the config file
    let config_path: String = if let Some(c) = matches.get_one::<String>("config") {
        c.to_string()
    } else {
        String::from("valentine.toml")
    };
    let buf = fs::read_to_string(&config_path).expect("Unable to open config file");
    let config: Config = toml::from_str(&buf).expect("Invalid config file");

    if let Some(matches) = matches.subcommand_matches("backup") {
        let file = matches.get_one::<String>("FILE").unwrap();
        cmd::backup::run(file);
    } else if let Some(matches) = matches.subcommand_matches("ssh") {
        cmd::ssh::run(config, matches);
    } else if let Some(_matches) = matches.subcommand_matches("web") {
        let config_path = PathBuf::from(config_path).canonicalize().unwrap();
        cmd::web::run(config, config_path);
    }
}
