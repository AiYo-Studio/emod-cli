use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use clap::ArgMatches;
use serde_json::Value;
use walkdir;
use zip::write::SimpleFileOptions;

use crate::utils::file::read_file_to_json;

struct ReleaseInfo {
    behavior_version: Vec<u32>,
    resource_version: Vec<u32>,
    behavior_identifier: String,
    resource_identifier: String,
}

pub fn execute(sub_matches: &ArgMatches, _: &PathBuf) {
    let project_dir = find_project_dir(sub_matches);
    if let Err(e) = project_dir {
        eprintln!("Error: Failed to find project directory: {}", e);
        return;
    }
    let project_dir = project_dir.unwrap();
    match get_current_release_info(&project_dir) {
        Ok(release_info) => {
            println!("🔖 当前行为包版本: {:?}", release_info.behavior_version);
            println!("🔖 当前资源包版本: {:?}", release_info.resource_version);
            if let Err(e) = release(sub_matches, &project_dir, &release_info) {
                eprintln!("❌ 组件打包失败: {}", e);
            } else {
                println!("🍀 组件打包完成");
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to get current release info: {}", e);
        }
    };
}

fn release(
    sub_matches: &ArgMatches,
    project_dir: &PathBuf,
    release_info: &ReleaseInfo,
) -> Result<(), std::io::Error> {
    let behavior_version = release_info.behavior_version.clone();
    let default_version = format!(
        "{}.{}.{}",
        behavior_version[0],
        behavior_version[1],
        behavior_version[2] + 1
    );
    let target = sub_matches
        .get_one::<String>("version")
        .unwrap_or(&default_version);
    println!("✨ 预计版本号: {}", target);
    // 转换版本号
    let version = target
        .split(".")
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<u32>, _>>()
        .unwrap();
    println!("📦 开始打包, 版本号: {:?}", &version);
    // 转换版本号
    let version = Value::Array(version.iter().map(|v| Value::from(*v)).collect());
    write_to_pack(&project_dir, &version)?;
    write_to_manifests(&project_dir, &release_info, &version)?;
    // 将 behavior_pack 和 resource_pack 打包到同一个 zip 中
    let output_path = package_folders(&project_dir, &release_info, &target)?;
    println!("📦 打包完成: {}", output_path);
    Ok(())
}

fn package_folders(
    project_dir: &PathBuf,
    release_info: &ReleaseInfo,
    target: &str,
) -> Result<String, std::io::Error> {
    let output_path = format!("{}/release_{}.zip", project_dir.display(), target);
    let file = fs::File::create(&output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let behavior_dir = format!(
        "{}/behavior_pack_{}",
        project_dir.display(),
        release_info.behavior_identifier
    );
    let resource_dir = format!(
        "{}/resource_pack_{}",
        project_dir.display(),
        release_info.resource_identifier
    );
    add_directory_to_zip(&mut zip, &project_dir, Path::new(&behavior_dir))?;
    add_directory_to_zip(&mut zip, &project_dir, Path::new(&resource_dir))?;
    zip.finish()?;
    Ok(output_path)
}

fn add_directory_to_zip(
    zip: &mut zip::ZipWriter<File>,
    project_dir: &PathBuf,
    src_dir: &Path,
) -> Result<(), std::io::Error> {
    if !src_dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{} is not a directory", src_dir.display()),
        ));
    }
    if get_directory_file_count(src_dir)? == 0 {
        return Ok(());
    }
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Bzip2)
        .unix_permissions(0o755);
    let mut buffer = Vec::new();
    for entry in walkdir::WalkDir::new(src_dir) {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(project_dir).unwrap();
        let path_as_string = name
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{:?} 不是有效的 UTF-8 路径", name),
                )
            })?
            .to_owned();
        if path.is_file() {
            if path_as_string.ends_with(".gitkeep") {
                continue;
            }
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() && get_directory_file_count(path)? > 0 {
            zip.add_directory(path_as_string, options)?;
        }
    }
    Ok(())
}

fn get_directory_file_count(src_dir: &Path) -> Result<usize, std::io::Error> {
    let mut count = 0;
    for entry in walkdir::WalkDir::new(src_dir) {
        let entry = entry?;
        if entry.path().is_file() && !entry.path().display().to_string().ends_with(".gitkeep") {
            count += 1;
        }
    }
    Ok(count)
}

fn write_to_pack(project_dir: &PathBuf, version: &Value) -> Result<(), std::io::Error> {
    let behavior_path = format!("{}/world_behavior_packs.json", project_dir.display());
    let resource_path = format!("{}/world_resource_packs.json", project_dir.display());
    let mut behavior_json = read_file_to_json(&PathBuf::from(&behavior_path))?;
    let mut resource_json = read_file_to_json(&PathBuf::from(&resource_path))?;

    behavior_json[0]["version"] = version.clone();
    resource_json[0]["version"] = version.clone();

    fs::write(
        &behavior_path,
        serde_json::to_string_pretty(&behavior_json)?,
    )?;
    fs::write(
        &resource_path,
        serde_json::to_string_pretty(&resource_json)?,
    )?;
    Ok(())
}

fn write_to_manifests(
    project_dir: &PathBuf,
    release_info: &ReleaseInfo,
    version: &Value,
) -> Result<(), std::io::Error> {
    let behavior_dir = format!(
        "{}/behavior_pack_{}",
        project_dir.display(),
        release_info.behavior_identifier
    );
    let resource_dir = format!(
        "{}/resource_pack_{}",
        project_dir.display(),
        release_info.resource_identifier
    );
    write_to_manifest(&PathBuf::from(&behavior_dir), &version)?;
    write_to_manifest(&PathBuf::from(&resource_dir), &version)?;
    Ok(())
}

fn write_to_manifest(project_dir: &PathBuf, version: &Value) -> Result<(), std::io::Error> {
    let manifest_path = format!("{}/pack_manifest.json", project_dir.display());
    let mut manifest_json = read_file_to_json(&PathBuf::from(&manifest_path))?;
    manifest_json["header"]["version"] = version.clone();
    manifest_json["modules"][0]["version"] = version.clone();
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest_json)?,
    )?;
    Ok(())
}

fn find_project_dir(sub_matches: &ArgMatches) -> Result<PathBuf, std::io::Error> {
    let default_target = String::from("./");
    let target = sub_matches
        .get_one::<String>("path")
        .unwrap_or(&default_target);
    Ok(PathBuf::from(target))
}

fn get_current_release_info(project_dir: &PathBuf) -> Result<ReleaseInfo, std::io::Error> {
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
