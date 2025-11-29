use crate::entity::project::ReleaseInfo;
use crate::error::Result;
use crate::utils::file::read_file_to_json;
use std::path::PathBuf;

pub mod project;

pub fn get_current_release_info(project_dir: &PathBuf) -> Result<ReleaseInfo> {
    let behavior_path = project_dir.join("world_behavior_packs.json");
    let resource_path = project_dir.join("world_resource_packs.json");

    let behavior_json = read_file_to_json(&behavior_path)?;
    let resource_json = read_file_to_json(&resource_path)?;

    let behavior_version = parse_version_array(&behavior_json[0]["version"])?;
    let resource_version = parse_version_array(&resource_json[0]["version"])?;

    let behavior_pack_uuid = behavior_json[0]["pack_id"]
        .as_str()
        .ok_or_else(|| crate::error::CliError::InvalidData("无效的 behavior pack_id".into()))?
        .to_string();

    let resource_pack_uuid = resource_json[0]["pack_id"]
        .as_str()
        .ok_or_else(|| crate::error::CliError::InvalidData("无效的 resource pack_id".into()))?
        .to_string();

    let behavior_identifier: String = behavior_pack_uuid.chars().take(8).collect();
    let resource_identifier: String = resource_pack_uuid.chars().take(8).collect();

    Ok(ReleaseInfo {
        behavior_version,
        resource_version,
        behavior_identifier,
        resource_identifier,
    })
}

fn parse_version_array(value: &serde_json::Value) -> Result<Vec<u32>> {
    value
        .as_array()
        .ok_or_else(|| crate::error::CliError::InvalidData("版本格式无效".into()))?
        .iter()
        .map(|v| {
            v.as_u64()
                .map(|n| n as u32)
                .ok_or_else(|| crate::error::CliError::InvalidData("版本号格式无效".into()))
        })
        .collect()
}
