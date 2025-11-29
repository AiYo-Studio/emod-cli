use crate::commands::ComponentsArgs;
use crate::entity;
use crate::utils::file;
use crate::error::Result;
use serde_json::{json, to_string_pretty};
use std::fs;
use std::path::PathBuf;

const COMPONENT_3D_ITEM: &str = "3ditem";

pub fn execute(args: &ComponentsArgs) {
    if let Err(e) = run_components(args) {
        eprintln!("错误: {}", e);
        return;
    }
    println!("成功: 组件已创建");
}

fn run_components(args: &ComponentsArgs) -> Result<()> {
    let project_path = file::find_project_dir(&args.path)?;
    
    validate_input_files(&args.geo, &args.texture)?;
    
    let identifier = args.identifier.as_deref().unwrap_or("unknown");
    
    match args.component.as_str() {
        COMPONENT_3D_ITEM => create_3dmodel(
            args.geo.as_deref().unwrap_or("./model.geo.json"),
            args.texture.as_deref().unwrap_or("./texture.png"),
            identifier,
            &project_path
        ),
        _ => Err(crate::error::CliError::NotFound(
            format!("组件 '{}' 不存在", args.component)
        )),
    }
}

fn validate_input_files(geo: &Option<String>, texture: &Option<String>) -> Result<()> {
    let geo_path = geo.as_deref().unwrap_or("./model.geo.json");
    let texture_path = texture.as_deref().unwrap_or("./texture.png");
    
    if !PathBuf::from(geo_path).exists() {
        return Err(crate::error::CliError::NotFound(
            format!("几何文件 {} 不存在", geo_path)
        ));
    }
    
    if !PathBuf::from(texture_path).exists() {
        return Err(crate::error::CliError::NotFound(
            format!("材质文件 {} 不存在", texture_path)
        ));
    }
    
    Ok(())
}

fn create_3dmodel(
    geo: &str,
    texture: &str,
    identifier: &str,
    project_path: &PathBuf,
) -> Result<()> {
    let project_info = entity::get_current_release_info(&project_path)?;
    
    let beh_path = project_path.join(format!(
        "behavior_pack_{}",
        project_info.behavior_identifier
    ));
    let res_path = project_path.join(format!(
        "resource_pack_{}",
        project_info.resource_identifier
    ));
    
    create_item_files(&beh_path, &res_path, identifier)?;
    copy_assets(&res_path, geo, texture, identifier)?;
    create_attachable_file(&res_path, identifier)?;
    
    Ok(())
}

fn create_item_files(beh_path: &PathBuf, res_path: &PathBuf, identifier: &str) -> Result<()> {
    let behavior_item = create_behavior_item_json(identifier);
    let resource_item = create_resource_item_json(identifier);
    
    let f_identifier = identifier.replace(":", "_");
    
    let items_beh_dir = beh_path.join("netease_items_beh");
    let items_res_dir = res_path.join("netease_items_res");
    
    fs::create_dir_all(&items_beh_dir)?;
    fs::create_dir_all(&items_res_dir)?;
    
    let beh_item_path = items_beh_dir.join(format!("{}.json", f_identifier));
    let res_item_path = items_res_dir.join(format!("{}.json", f_identifier));
    
    fs::write(&beh_item_path, to_string_pretty(&behavior_item)?)?;
    fs::write(&res_item_path, to_string_pretty(&resource_item)?)?;
    
    Ok(())
}

fn create_behavior_item_json(identifier: &str) -> serde_json::Value {
    json!({
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
    })
}

fn create_resource_item_json(identifier: &str) -> serde_json::Value {
    json!({
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
    })
}

fn copy_assets(
    res_path: &PathBuf,
    geo: &str,
    texture: &str,
    identifier: &str,
) -> Result<()> {
    let f_identifier = identifier.replace(":", "_");
    
    copy_texture(res_path, texture, &f_identifier)?;
    copy_geometry(res_path, geo, identifier, &f_identifier)?;
    
    Ok(())
}

fn copy_texture(res_path: &PathBuf, texture: &str, f_identifier: &str) -> Result<()> {
    let texture_dir = res_path.join("textures/models");
    fs::create_dir_all(&texture_dir)?;
    
    let target_texture = texture_dir.join(format!("{}.png", f_identifier));
    fs::copy(texture, target_texture)?;
    
    Ok(())
}

fn copy_geometry(
    res_path: &PathBuf,
    geo: &str,
    identifier: &str,
    f_identifier: &str,
) -> Result<()> {
    let geo_dir = res_path.join("models/entity");
    fs::create_dir_all(&geo_dir)?;
    
    let mut geo_value = file::read_file_to_json(&PathBuf::from(geo))?;
    
    let geo_name = format!("geometry.{}", identifier.replace(":", "."));
    geo_value["format_version"] = json!("1.12.0");
    geo_value["minecraft:geometry"][0]["description"]["identifier"] = json!(geo_name);
    
    let target_geo = geo_dir.join(format!("{}.geo.json", f_identifier));
    file::write_json_to_file(&target_geo, &geo_value)?;
    
    Ok(())
}

fn create_attachable_file(res_path: &PathBuf, identifier: &str) -> Result<()> {
    let attachable_dir = res_path.join("attachables");
    fs::create_dir_all(&attachable_dir)?;
    
    let f_identifier = identifier.replace(":", "_");
    let geo_name = identifier.replace(":", ".");
    
    let attachable = json!({
        "format_version": "1.10.0",
        "minecraft:attachable": {
            "description": {
                "geometry": {
                    "default": format!("geometry.{}", &geo_name)
                },
                "identifier": identifier,
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
    
    let target_file = attachable_dir.join(format!("{}.json", &f_identifier));
    fs::write(target_file, to_string_pretty(&attachable)?)?;
    
    Ok(())
}
