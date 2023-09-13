use {Config, Context};
use routes::*;

use hayaku::{Http, Router};
use r2d2;
use r2d2_sqlite::SqliteConnectionManager;

use std::{env, fs, process};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn run(config: Config, config_path: PathBuf) {
    info!("Starting up server");

    // Create db connection pool
    let manager = SqliteConnectionManager::file(config.db_path);
    let pool = r2d2::Pool::new(manager).expect("Failed to create pool");

    {
        // Run migrations
        use rusqlite_migration::{M, Migrations};
        let migrations = Migrations::new(vec![
            //M::up(include_str!("../migrations/1/up.sql"))
            //    .down(include_str!("../migrations/1/down.sql")),
        ]);
        let mut conn = pool.get().unwrap();
        info!("Running migrations");
        migrations.to_latest(&mut conn).unwrap();
    }

    // Create repository folder
    {
        let path = &config.repo_dir;
        if !path.exists() {
            fs::create_dir(path).unwrap();
        } else if !path.is_dir() {
            println!("unable to create repository folder, file already exists!");
            process::exit(1);
        }
    }

    let ssh_dir = config.ssh_dir.unwrap_or_else(|| {
        let mut home = ::dirs::home_dir().unwrap();
        home.push(".ssh");
        home
    });

    if !ssh_dir.exists() {
        fs::create_dir_all(&ssh_dir).unwrap();
    }

    let mount = match config.mount {
        Some(m) => if m.ends_with('/') {
            m
        } else {
            m + "/"
        },
        None => "/".to_string(),
    };

    let url = match config.url {
        Some(mut m) => if m.ends_with('/') {
            m.pop();
            m
        } else {
            m
        },
        None => "http://localhost".to_string(),
    };

    let sessions = if config.sessions_dir.exists() && config.sessions_dir.is_dir() {
        ::sessions::SessionSet::load(config.sessions_dir).expect("failed to load sessions")
    } else {
        ::sessions::SessionSet::new(config.sessions_dir).expect("failed to create sessions")
    };

    let ctx = Context {
        db_pool: pool,
        mount: mount,
        logins: Arc::new(Mutex::new(sessions)),
        name: config.name.unwrap_or_else(|| String::from("Valentine")),
        url: url,
        ssh: config.ssh,
        signup: config.signup.unwrap_or(false),
        repo_dir: config.repo_dir,
        ssh_dir: ssh_dir,
        bin_path: env::current_exe().unwrap(),
        config_path: config_path,
    };

    let mut router = Router::mount(ctx.mount.clone());
    router.set_not_found_handler(Arc::new(not_found));
    router.set_internal_error_handler(Arc::new(internal_error));
    router.static_file_handle("/resources", "resources");
    router!{
        router,
        get "/" => home,
        get "/explore" => explore,
        get "/{user}" => user,
        get "/{user}/{repo}" => repo::view,
        get "/{user}/{repo}/log" => repo::log_default,
        get "/{user}/{repo}/log/{id}" => repo::log,
        get "/{user}/{repo}/refs" => repo::refs_list,
        get "/{user}/{repo}/refs/{id}" => repo::commit,
        get "/{user}/{repo}/refs/{id}/{*filepath}" => repo::src,
        get "/{user}/{repo}/refs/{id}/raw/{*filepath}" => repo::raw,

        // Git pull
        // TODO: use regex to assert that `repo` ends with .git
        get "/{user}/{repo}/info/refs" => git_routes::pull_handshake,
        post "/{user}/{repo}/git-upload-pack" => git_routes::pull,

        // User
        get "/signup" => user::signup,
        post "/signup" => user::signup_post,
        get "/login" => user::login,
        post "/login" => user::login_post,
        get "/logout" => user::logout,
        get "/settings" => user::settings,
        post "/settings/add-ssh-key" => user::add_ssh_key,
        get r"/settings/delete-ssh-key/{id:\d+}" => user::delete_ssh_key,
        get "/repo/new" => user::repo::new,
        post "/repo/new" => user::repo::new_post,
        get "/{user}/{repo}/settings" => user::repo::settings,
        post "/{user}/{repo}/settings/name" => user::repo::settings_name,
        post "/{user}/{repo}/settings/delete" => user::repo::delete,
    }

    let addr = config.addr.unwrap_or_else(|| "127.0.0.1:3000".parse().unwrap());
    info!("running server at {}", addr);
    Http::new(router, ctx).listen_and_serve(addr);
}
