use std::{env, fs, io, path::PathBuf};

use clap::{arg, Command};
use reqwest::blocking::Client;

/**
 * 调试项目，请勿使用！
 */
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
        .arg_required_else_help(true)
        .get_matches();
    let temp_dir = check_temp_dir();
    match matches.subcommand() {
        Some(("create", sub_matches)) => {
            let default_target = String::from("default");
            let name = sub_matches.get_one::<String>("name").unwrap();
            let target = sub_matches
                .get_one::<String>("target")
                .unwrap_or(&default_target);
            create_project(name, target, &temp_dir);
        }
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

// TODO: 待分离代码结构
fn create_project(name: &str, target: &str, temp_dir: &PathBuf) {
    // 检查目标示例是否存在
    let check_url = format!(
        "https://api.github.com/repos/AiYo-Studio/emod-cli/contents/examples/{}",
        target
    );
    // TODO: 代理调试, 待重构
    let proxy = reqwest::Proxy::https("http://127.0.0.1:1080").unwrap();
    let client = Client::builder().proxy(proxy).build().unwrap();
    let response = client
        .get(check_url)
        .header("User-Agent", "emod-cli")
        .send();
    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                eprintln!("Error: Example target '{}' does not exist", target);
                return;
            }
            // 创建本地目录
            let local_dir = format!("./{}", name);
            let _ = fs::create_dir(&local_dir);
            // 克隆远程仓库
            let url = format!("https://github.com/AiYo-Studio/emod-cli.git");
            let target = PathBuf::from(format!("{}/examples/{}", temp_dir.display(), target));
            match clone_remote_project(url, temp_dir) {
                Ok(_) => match copy_folder(&target, &PathBuf::from(local_dir)) {
                    Ok(_) => println!("Success: Project created"),
                    Err(e) => eprintln!("Error: Failed to copy folder: {}", e),
                },
                Err(e) => eprintln!("Error: Failed to clone remote project: {}", e),
            };
        }
        Err(e) => {
            eprintln!("Error checking example target: {}", e);
        }
    }
}

fn clone_remote_project(url: String, temp_dir: &PathBuf) -> io::Result<()> {
    std::process::Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(format!("{}", temp_dir.display()))
        .output()?;
    Ok(())
}

fn copy_folder(src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
    if !src.exists() || !src.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Source directory not found",
        ));
    }
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(src_path.file_name().unwrap());
        if src_path.is_file() {
            fs::copy(&src_path, &dest_path)?;
        } else if src_path.is_dir() {
            copy_folder(&src_path, &dest_path)?;
        }
    }

    Ok(())
}
