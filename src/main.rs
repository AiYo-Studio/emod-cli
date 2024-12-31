mod utils {
    pub mod file;
    pub mod git;
}
mod commands {
    pub mod create;
    pub mod release;
}

use std::{env, fs, path::PathBuf};

use clap::{arg, Command};

fn main() {
    let matches = Command::new("emod-cli")
        .version("1.0.0")
        .author("AiYo Studio")
        .about("Convenient Management of NetEase Minecraft Mod Project")
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("create")
                .arg(arg!(-n --name <name> "The name of the mod").required(true))
                .arg(arg!(-t --target [target] "Example target, default example is 'default'"))
                .about("Create a new mod project")
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("release")
                .arg(arg!(-p --path <path> "The path of the project").required(false))
                .arg(arg!(-v --version <version> "The version of the project").required(false))
                .about("Package project"),
        )
        .arg_required_else_help(true)
        .get_matches();
    let temp_dir = check_temp_dir();
    match matches.subcommand() {
        Some(("create", sub_matches)) => commands::create::execute(sub_matches, &temp_dir),
        Some(("release", sub_matches)) => commands::release::execute(sub_matches, &temp_dir),
        _ => {
            unreachable!();
        }
    }
}

fn check_temp_dir() -> PathBuf {
    let mut temp_dir = env::temp_dir();
    temp_dir.push("emod-cli");
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        eprintln!("Error: Failed to create temp directory: {}", e);
    }
    temp_dir
}
