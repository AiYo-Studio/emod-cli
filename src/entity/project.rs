pub struct ProjectInfo {
    pub name: String,
    pub lower_name: String,
    pub behavior_pack_uuid: String,
    pub resource_pack_uuid: String,
    pub behavior_module_uuid: String,
    pub resource_module_uuid: String,
}

pub struct ReleaseInfo {
    pub behavior_version: Vec<u32>,
    pub resource_version: Vec<u32>,
    pub behavior_identifier: String,
    pub resource_identifier: String,
}