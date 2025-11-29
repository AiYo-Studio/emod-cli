use crate::{
    config::Config,
    entity::project::ProjectInfo,
    utils::{file, http::HttpClient},
};
use std::{fs, path::PathBuf};

use crate::commands::CreateArgs;
use crate::error::Result;
use crate::utils::git;
use uuid::Uuid;

pub fn execute(args: &CreateArgs, temp_dir: &PathBuf) {
    if let Err(e) = create_project(&args.name, args.target.as_deref(), temp_dir) {
        eprintln!("错误: {}", e);
        return;
    }
    println!("成功: 项目已创建");
}

fn create_project(name: &str, target: Option<&str>, temp_dir: &PathBuf) -> Result<()> {
    let target = target.unwrap_or("default");

    check_example_exists(target)?;

    let local_dir = PathBuf::from(format!("./{}", name));
    fs::create_dir(&local_dir)?;

    clone_and_copy_template(target, temp_dir, &local_dir)?;

    initialize_project(&local_dir, name)?;

    Ok(())
}

fn check_example_exists(target: &str) -> Result<()> {
    let check_url = format!(
        "https://api.github.com/repos/AiYo-Studio/emod-cli/contents/examples/{}",
        target
    );

    let client = if cfg!(debug_assertions) {
        HttpClient::new_with_proxy("http://127.0.0.1:1080")?
    } else {
        HttpClient::new()?
    };

    let resp = client.get(&check_url)?;

    if !resp.status().is_success() {
        return Err(crate::error::CliError::NotFound(format!(
            "示例模板 '{}' 不存在",
            target
        )));
    }

    Ok(())
}

fn clone_and_copy_template(target: &str, temp_dir: &PathBuf, local_dir: &PathBuf) -> Result<()> {
    let _ = fs::remove_dir_all(format!("{}/tmp", temp_dir.display()));

    let config = Config::load();
    let url = &config.repo_url;
    git::clone_remote_project(url.to_string(), temp_dir)?;

    let target_dir = PathBuf::from(format!("{}/tmp/examples/{}", temp_dir.display(), target));
    file::copy_folder(&target_dir, local_dir)?;

    Ok(())
}

fn initialize_project(local_dir: &PathBuf, name: &str) -> Result<()> {
    let lower_name = format!(
        "{}{}",
        name.chars().next().unwrap().to_lowercase(),
        &name[1..]
    );

    println!("项目名称: {}", name);
    println!("标识名称: {}", lower_name);

    let scripts_dir = local_dir.join(format!("behavior_pack/{}Scripts", lower_name));
    fs::rename(local_dir.join("behavior_pack/exampleScripts"), &scripts_dir)?;

    let project_info = generate_project_info(name, &lower_name);

    apply_project_info(local_dir, &scripts_dir, &project_info)?;

    rename_pack_folders(local_dir, &project_info)?;

    Ok(())
}

fn generate_project_info(name: &str, lower_name: &str) -> ProjectInfo {
    ProjectInfo {
        name: name.to_string(),
        lower_name: lower_name.to_string(),
        behavior_pack_uuid: Uuid::new_v4().to_string(),
        resource_pack_uuid: Uuid::new_v4().to_string(),
        behavior_module_uuid: Uuid::new_v4().to_string(),
        resource_module_uuid: Uuid::new_v4().to_string(),
    }
}

fn apply_project_info(
    local_dir: &PathBuf,
    scripts_dir: &PathBuf,
    info: &ProjectInfo,
) -> Result<()> {
    let manifest_files = vec![
        local_dir.join("world_behavior_packs.json"),
        local_dir.join("world_resource_packs.json"),
        local_dir.join("behavior_pack/pack_manifest.json"),
        local_dir.join("resource_pack/pack_manifest.json"),
    ];

    for path in manifest_files {
        apply_info_to_json(&path, info)?;
    }

    process_python_files(scripts_dir, info)?;

    Ok(())
}

fn apply_info_to_json(path: &PathBuf, info: &ProjectInfo) -> Result<()> {
    println!(" - 修改文件: {}", path.display());
    file::update_json_file(path, |json| {
        let content = serde_json::to_string(json)?;
        let updated = content
            .replace("{behavior_pack_uuid}", &info.behavior_pack_uuid)
            .replace("{resource_pack_uuid}", &info.resource_pack_uuid)
            .replace("{behavior_module_uuid}", &info.behavior_module_uuid)
            .replace("{resource_module_uuid}", &info.resource_module_uuid)
            .replace("__mod_name__", &info.name)
            .replace("__mod_name_lower__", &info.lower_name);
        *json = serde_json::from_str(&updated)?;
        Ok(())
    })
}

fn process_python_files(dir: &PathBuf, info: &ProjectInfo) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            process_python_files(&entry.path(), info)?;
        }
    } else if dir.extension().and_then(|s| s.to_str()) == Some("py") {
        apply_info_to_python(dir, info)?;
    }

    Ok(())
}

fn apply_info_to_python(path: &PathBuf, info: &ProjectInfo) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let updated = content
        .replace("__mod_name__", &info.name)
        .replace("__mod_name_lower__", &info.lower_name);
    fs::write(path, updated)?;
    Ok(())
}

fn rename_pack_folders(local_dir: &PathBuf, info: &ProjectInfo) -> Result<()> {
    let behavior_suffix: String = info.behavior_pack_uuid.chars().take(8).collect();
    let resource_suffix: String = info.resource_pack_uuid.chars().take(8).collect();

    fs::rename(
        local_dir.join("behavior_pack"),
        local_dir.join(format!("behavior_pack_{}", behavior_suffix)),
    )?;
    fs::rename(
        local_dir.join("resource_pack"),
        local_dir.join(format!("resource_pack_{}", resource_suffix)),
    )?;

    Ok(())
}