use crate::entity::project::ReleaseInfo;
use crate::utils::file::read_file_to_json;
use std::path::PathBuf;

pub mod project;

pub fn get_current_release_info(project_dir: &PathBuf) -> anyhow::Result<ReleaseInfo> {
    let behavior_path = format!("{}/world_behavior_packs.json", project_dir.display());
    let resource_path = format!("{}/world_resource_packs.json", project_dir.display());
    let behavior_json = read_file_to_json(&PathBuf::from(behavior_path))?;
    let resource_json = read_file_to_json(&PathBuf::from(resource_path))?;
    let behavior_version = behavior_json[0]["version"].as_array().unwrap();
    let resource_version = resource_json[0]["version"].as_array().unwrap();
    // 定义发布信息
    let behavior_version = behavior_version
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect::<Vec<u32>>();
    let resource_version = resource_version
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect::<Vec<u32>>();
    let behavior_pack_uuid = behavior_json[0]["pack_id"].as_str().unwrap().to_string();
    let resource_pack_uuid = resource_json[0]["pack_id"].as_str().unwrap().to_string();
    let behavior_identifier = behavior_pack_uuid.chars().take(8).collect::<String>();
    let resource_identifier = resource_pack_uuid.chars().take(8).collect::<String>();
    Ok(ReleaseInfo {
        behavior_version,
        resource_version,
        behavior_identifier,
        resource_identifier,
    })
}
