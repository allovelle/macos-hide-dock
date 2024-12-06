#![allow(clippy::unit_arg)]

#[cfg(not(target_os = "macos"))]
const CRATE_MACOS_ONLY: () = panic!("Crate works only on MacOS");

use std::os::unix::fs::PermissionsExt;
use std::process::{self, Command};
use std::{collections::HashMap, env, fs, io, path, str};

#[derive(Debug)]
pub enum Error
{
    Msg(&'static str),
    Io(io::Error),
    Var(env::VarError),
}

fn main() -> Result<(), Error>
{
    if let Some(cmd) = env::args().nth(1)
    {
        let path = &std::path::Path::new("/Users")
            .join(env::var("USER").or(env::var("LOGNAME"))?)
            .join("Library/Application Support/xbar/plugins")
            .join("toggle-dock.1m.sh");

        if cmd == "install"
        {
            fs::write(path, include_str!("toggle-dock.sh"))?;
            make_executable(path)?;
            println!("Installed script\nRefreshing plugins...");
            return refresh_xbar_plugins().map_err(Into::into);
        }
        else if cmd == "uninstall"
        {
            fs::remove_file(path)?;
            println!("Uninstalled script\nRefreshing plugins...");
            return refresh_xbar_plugins().map_err(Into::into);
        }
        else
        {
            let pkg = env!("CARGO_PKG_NAME");
            return Ok(println!("Usage: {} [install | uninstall]", pkg));
        }
    }

    let command = &["defaults", "read", "com.apple.dock", "autohide"];

    let is_hidden_output = &run_command(command)?.stdout;
    let bools = HashMap::from([("0", false), ("1", true)]);
    let autohide_enabled: bool =
        bools[String::from_utf8_lossy(is_hidden_output).trim()];
    let toggled = (!autohide_enabled).to_string();

    let commands = [
        &["defaults", "write", "com.apple.dock", "autohide", "-bool", &toggled],
        &["killall", "Dock"][..],
    ];

    for command in commands
    {
        if !run_command(command)?.status.success()
        {
            return Err(Error::Msg("Something went wrong"));
        }
    }

    Ok(println!("Successfully toggled Dock"))
}

fn make_executable(path: &path::Path) -> Result<(), io::Error>
{
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}

fn refresh_xbar_plugins() -> Result<(), std::io::Error>
{
    let cmd = "xbar://app.xbarapp.com/refreshAllPlugins";
    Command::new("open").arg(cmd).status().map(drop)
}

fn run_command(cli: &[&str]) -> Result<process::Output, io::Error>
{
    Command::new(cli[0]).args(&cli[1 ..]).output()
}

impl From<io::Error> for Error
{
    #[rustfmt::skip]
    fn from(value: io::Error) -> Self { Self::Io(value) }
}

impl From<env::VarError> for Error
{
    #[rustfmt::skip]
    fn from(value: env::VarError) -> Self { Self::Var(value) }
}