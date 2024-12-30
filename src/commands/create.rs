use std::{
    fs, io,
    path::{Path, PathBuf},
};

use clap::ArgMatches;
use reqwest::blocking::Client;
use uuid::Uuid;

use crate::utils::{file, git};

struct ProjectInfo {
    name: String,
    lower_name: String,
    behavior_pack_uuid: String,
    resource_pack_uuid: String,
    behavior_module_uuid: String,
    resource_module_uuid: String,
}

pub fn execute(sub_matches: &ArgMatches, temp_dir: &PathBuf) {
    let default_target = String::from("default");
    let name = sub_matches.get_one::<String>("name").unwrap();
    let target = sub_matches
        .get_one::<String>("target")
        .unwrap_or(&default_target);
    create_project(name, target, &temp_dir);
}

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
    let resp = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error checking example target: {}", e);
            return;
        }
    };

    if !resp.status().is_success() {
        eprintln!("Error: Example target '{}' does not exist", target);
        return;
    }
    // 删除缓存目录重新拉取
    let _ = fs::remove_dir_all(format!("{}/tmp", temp_dir.display()));
    // 创建本地目录
    let local_dir = format!("./{}", name);
    if let Err(e) = fs::create_dir(&local_dir) {
        eprintln!("Error: Failed to create directory: {}", e);
        return;
    }

    // 克隆远程仓库
    let url = format!("https://github.com/AiYo-Studio/emod-cli.git");
    let target_dir = PathBuf::from(format!("{}/tmp/examples/{}", temp_dir.display(), target));

    if let Err(e) = git::clone_remote_project(url, temp_dir) {
        eprintln!("Error: Failed to clone remote project: {}", e);
        return;
    }

    if let Err(e) = file::copy_folder(&target_dir, &PathBuf::from(&local_dir)) {
        eprintln!("Error: Failed to copy folder: {}", e);
        return;
    }

    if let Err(e) = initialize_project(&local_dir, &name) {
        eprintln!("Error: Failed to initialize project: {}", e);
        return;
    }

    println!("Success: Project created");
}

fn initialize_project(local_dir: &str, name: &str) -> io::Result<()> {
    // 将 name 首字母小写后返回新的 name
    let first_lower_name = name.to_lowercase().chars().next().unwrap();
    let new_name = format!("{}{}", first_lower_name, &name[1..]);
    println!("Project Name: {}", name);
    println!("Identify Name: {}", new_name);
    // 最终目标目录
    let final_dir = format!("{}/behavior_pack/{}Scripts", local_dir, new_name);
    // 重命名基础文件夹
    fs::rename(
        format!("{}/behavior_pack/exampleScripts", local_dir),
        &final_dir,
    )?;
    // 生成项目信息
    let behavior_pack_uuid = Uuid::new_v4().to_string();
    let resource_pack_uuid = Uuid::new_v4().to_string();
    let behavior_module_uuid = Uuid::new_v4().to_string();
    let resource_module_uuid = Uuid::new_v4().to_string();
    let info = ProjectInfo {
        name: name.to_string(),
        lower_name: new_name,
        behavior_pack_uuid,
        resource_pack_uuid,
        behavior_module_uuid,
        resource_module_uuid,
    };
    // 修改 world_behavior_pack.json 和 world_resource_pack.json 中 UUID
    let apply_uuid_list = vec![
        format!("{}/world_behavior_packs.json", &local_dir),
        format!("{}/world_resource_packs.json", &local_dir),
        format!("{}/behavior_pack/pack_manifest.json", &local_dir),
        format!("{}/resource_pack/pack_manifest.json", &local_dir),
    ];
    for ele in apply_uuid_list {
        apply_info_uuid(ele, &info)?;
    }
    // 替换目标目录中 python 文件中的变量
    search_python_file(final_dir.clone(), &info);
    Ok(())
}

fn search_python_file(dir: String, info: &ProjectInfo) {
    let path = Path::new(&dir);
    if path.is_dir() {
        let entries = fs::read_dir(path);
        if entries.is_err() {
            return;
        }
        for entry in entries.unwrap() {
            if entry.is_err() {
                return;
            }
            let entry = entry.unwrap();
            search_python_file(entry.path().display().to_string(), info);
        }
    } else if path.file_name().unwrap().to_str().unwrap().ends_with(".py") {
        match apply_info_code(dir, &info) {
            Ok(_) => (),
            Err(e) => eprintln!("Error: Failed to apply info code: {}", e),
        }
    }
}

fn apply_info_code(dir: String, info: &ProjectInfo) -> io::Result<()> {
    let content = fs::read_to_string(&dir)?;
    let final_content = content
        .replace("__mod_name__", &info.name)
        .replace("__mod_name_lower__", &info.lower_name);
    fs::write(dir, final_content)?;
    Ok(())
}

fn apply_info_uuid(dir: String, info: &ProjectInfo) -> io::Result<()> {
    println!(" - Modify File: {}", &dir);
    match fs::read_to_string(&dir) {
        Ok(content) => {
            let final_content = content
                .replace("{behavior_pack_uuid}", &info.behavior_pack_uuid)
                .replace("{resource_pack_uuid}", &info.resource_pack_uuid)
                .replace("{behavior_module_uuid}", &info.behavior_module_uuid)
                .replace("{resource_module_uuid}", &info.resource_module_uuid)
                .replace("__mod_name__", &info.name)
                .replace("__mod_name_lower__", &info.lower_name);
            fs::write(dir, final_content)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: Failed to read file: {}", e);
            Err(e)
        }
    }
}
