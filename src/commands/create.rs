use crate::{
    config::Config,
    entity::project::ProjectInfo,
    template::TemplateEngine,
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

    let template_dir = clone_and_copy_template(target, temp_dir, &local_dir)?;

    initialize_project_with_template(&template_dir, &local_dir, name)?;

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

fn clone_and_copy_template(
    target: &str,
    temp_dir: &PathBuf,
    local_dir: &PathBuf,
) -> Result<PathBuf> {
    let _ = fs::remove_dir_all(format!("{}/tmp", temp_dir.display()));

    let config = Config::load();
    let url = &config.repo_url;
    git::clone_remote_project(url.to_string(), temp_dir)?;

    let target_dir = PathBuf::from(format!("{}/tmp/examples/{}", temp_dir.display(), target));
    file::copy_folder(&target_dir, local_dir)?;

    Ok(target_dir)
}

fn initialize_project_with_template(
    template_dir: &PathBuf,
    local_dir: &PathBuf,
    name: &str,
) -> Result<()> {
    let lower_name = format!(
        "{}{}",
        name.chars().next().unwrap().to_lowercase(),
        &name[1..]
    );

    println!("项目名称: {}", name);
    println!("标识名称: {}", lower_name);

    let project_info = generate_project_info(name, &lower_name);

    let mut engine = TemplateEngine::load(template_dir)?;

    engine.set_variable("mod_name".to_string(), project_info.name.clone());
    engine.set_variable(
        "mod_name_lower".to_string(),
        project_info.lower_name.clone(),
    );
    engine.set_variable(
        "behavior_pack_uuid".to_string(),
        project_info.behavior_pack_uuid.clone(),
    );
    engine.set_variable(
        "resource_pack_uuid".to_string(),
        project_info.resource_pack_uuid.clone(),
    );
    engine.set_variable(
        "behavior_module_uuid".to_string(),
        project_info.behavior_module_uuid.clone(),
    );
    engine.set_variable(
        "resource_module_uuid".to_string(),
        project_info.resource_module_uuid.clone(),
    );
    engine.set_variable(
        "behavior_pack_uuid_short".to_string(),
        project_info.behavior_pack_uuid.chars().take(8).collect(),
    );
    engine.set_variable(
        "resource_pack_uuid_short".to_string(),
        project_info.resource_pack_uuid.chars().take(8).collect(),
    );

    engine.process_directory(local_dir)?;

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
