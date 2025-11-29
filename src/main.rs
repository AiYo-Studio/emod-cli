mod commands;
mod entity;
mod utils;
mod error;
mod config;
mod template;

use crate::commands::{Cli, Commands};
use clap::Parser;
use std::{env, fs, path::PathBuf};

fn main() {
    let cli = Cli::parse();
    let temp_dir = check_temp_dir();
    match &cli.command {
        Commands::Release(args) => commands::release::execute(args),
        Commands::Create(args) => commands::create::execute(args, &temp_dir),
        Commands::Components(args) => commands::components::execute(args),
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