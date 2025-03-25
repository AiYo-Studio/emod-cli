use crate::commands::ComponentsArgs;
use crate::entity;
use crate::utils::file;
use anyhow::Context;
use serde_json::{json, to_string_pretty};
use std::fs;
use std::path::PathBuf;

const COMPONENT_3D_ITEM: &str = "3ditem";

pub fn execute(args: &ComponentsArgs) {
    let component = &args.component;
    let path = file::find_project_dir(&args.path).unwrap();
    let geo = args.geo.as_deref().unwrap_or("./model.geo.json");
    let geo_buf = PathBuf::from(geo);
    if !geo_buf.exists() {
        eprintln!("Error: Not found the geometry file {}", geo);
        return;
    }
    let texture = args.texture.as_deref().unwrap_or("./texture.png");
    let texture_buf = PathBuf::from(texture);
    if !texture_buf.exists() {
        eprintln!("Error: Not found the texture file {}", texture);
        return;
    }
    let identifier = args.identifier.as_deref().unwrap_or("unknown");
    match component.as_str() {
        COMPONENT_3D_ITEM => create_3dmodel(geo, texture, identifier, &path)
            .expect("Failed to create 3ditem component"),
        _ => eprintln!("Error: Not found the component {}", component),
    }
}

fn create_3dmodel(
    geo: &str,
    texture: &str,
    identifier: &str,
    project_path: &PathBuf,
) -> anyhow::Result<()> {
    // 获取项目路径
    let project_info = entity::get_current_release_info(&project_path).unwrap();
    // 获取行为文件目录
    let beh_path = project_path.join(format!(
        "behavior_pack_{}",
        project_info.behavior_identifier
    ));
    // 获取资源文件目录
    let res_path = project_path.join(format!(
        "resource_pack_{}",
        project_info.resource_identifier
    ));
    create_item(&beh_path, &res_path, identifier)
        .with_context(|| format!("Failed to create item file: {}", &identifier))?;
    copy_geometry_and_texture(&res_path, &geo, &texture, &identifier)
        .with_context(|| format!("Failed to copy geo and texture file: {}", &identifier))?;
    create_attachable(&res_path, &identifier)
        .with_context(|| format!("Failed to create attachable file: {}", &identifier))?;
    Ok(())
}

fn create_item(beh_path: &PathBuf, res_path: &PathBuf, identifier: &str) -> anyhow::Result<()> {
    // 预设物品行为文件
    let netease_item_beh = json!({
        "format_version": "1.10",
        "minecraft:item": {
            "components": {
                "minecraft:max_damage": 10,
                "netease:armor": {
                    "armor_slot": 3,
                    "defense": 20,
                    "enchantment": 10
                }
            },
            "description": {
                "category": "Equipment",
                "identifier": identifier,
                "register_to_create_menu": true
            }
        }
    });
    // 预设物品资源文件
    let netease_item_res = json!({
        "format_version": "1.10",
        "minecraft:item": {
            "components": {
                "minecraft:icon": identifier
            },
            "description": {
                "category": "Equipment",
                "identifier": identifier,
                "register_to_create_menu": true
            }
        }
    });
    // 预设目标信息与文件
    let f_identifier = identifier.replace(":", "_");
    let items_beh_dir = beh_path.join("netease_items_beh");
    let items_res_dir = res_path.join("netease_items_res");
    let beh_item_path = items_beh_dir.join(format!("{}.json", f_identifier));
    let res_item_path = items_res_dir.join(format!("{}.json", f_identifier));
    if !items_beh_dir.exists() {
        fs::create_dir_all(&items_beh_dir)?;
    }
    if !items_res_dir.exists() {
        fs::create_dir_all(&items_res_dir)?;
    }
    let pretty_beh = to_string_pretty(&netease_item_beh)?;
    let pretty_res = to_string_pretty(&netease_item_res)?;
    fs::write(&beh_item_path, pretty_beh)?;
    fs::write(&res_item_path, pretty_res)?;
    Ok(())
}

fn copy_geometry_and_texture(
    res_path: &PathBuf,
    geo: &str,
    texture: &str,
    identifier: &str,
) -> anyhow::Result<()> {
    let f_identifier = identifier.replace(":", "_");
    // 复制材质文件
    let texture_dir = res_path.join("textures/models");
    if !texture_dir.exists() {
        fs::create_dir_all(&texture_dir)
            .with_context(|| format!("Failed to create directory: {}", &texture_dir.display()))?;
    }
    let target_texture = &texture_dir.join(format!("{}.png", f_identifier));
    fs::copy(texture, target_texture)?;
    // 复制几何文件
    let geo_dir = res_path.join("models/entity");
    if !geo_dir.exists() {
        fs::create_dir_all(&geo_dir)
            .with_context(|| format!("Failed to create directory: {}", &geo_dir.display()))?;
    }
    let target_geo = &geo_dir.join(format!("{}.geo.json", f_identifier));
    let mut value = file::read_file_to_json(&PathBuf::from(geo))
        .with_context(|| format!("Failed to load geo file: {}", &geo))?;
    let geo_name = format!("geometry.{}", identifier.replace(":", "."));
    value["format_version"] = json!("1.12.0");
    value["minecraft:geometry"][0]["description"]["identifier"] = json!(geo_name);
    let pretty_geo = to_string_pretty(&value)?;
    fs::write(&target_geo, &pretty_geo)
        .with_context(|| format!("Failed to write geo file: {}", target_geo.display()))?;
    Ok(())
}

fn create_attachable(res_path: &PathBuf, identifier: &str) -> anyhow::Result<()> {
    let attachable_dir = res_path.join("attachables");
    if !attachable_dir.exists() {
        fs::create_dir_all(&attachable_dir).with_context(|| {
            format!(
                "Failed to create attachables folder: {}",
                attachable_dir.display()
            )
        })?
    }
    let f_identifier = identifier.replace(":", "_");
    let geo_name = identifier.replace(":", ".");
    let target_file = attachable_dir.join(format!("{}.json", &f_identifier));
    let attachable_temp = json!({
        "format_version": "1.10.0",
        "minecraft:attachable": {
            "description": {
                "geometry": {
                    "default": format!("geometry.{}", &geo_name)
                },
                "identifier": &identifier,
                "materials": {
                    "default": "armor",
                    "enchanted": "armor_enchanted"
                },
                "render_controllers": [
                    "controller.render.armor"
                ],
                "scripts": {
                    "parent_setup": "variable.chest_layer_visible = 0.0;"
                },
                "textures": {
                    "default": format!("textures/models/{}", &f_identifier),
                    "enchanted": "textures/misc/enchanted_item_glint"
                }
            }
        }
    });
    fs::write(target_file, to_string_pretty(&attachable_temp)?)?;
    Ok(())
}
