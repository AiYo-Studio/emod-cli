use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde_json::Value;
use walkdir;
use zip::write::SimpleFileOptions;

use crate::commands::ReleaseArgs;
use crate::utils::file;
use crate::{entity, entity::project::ReleaseInfo, utils::file::read_file_to_json};
use anyhow::{anyhow, Result};

pub fn execute(args: &ReleaseArgs) {
    let project_dir = match file::find_project_dir(&args.path) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error: Failed to find project directory: {:#}", e);
            return;
        }
    };
    match entity::get_current_release_info(&project_dir) {
        Ok(release_info) => {
            println!("🔖 当前行为包版本: {:?}", release_info.behavior_version);
            println!("🔖 当前资源包版本: {:?}", release_info.resource_version);
            if let Err(e) = release(&args.ver, &project_dir, &release_info) {
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
    version: &Option<String>,
    project_dir: &PathBuf,
    release_info: &ReleaseInfo,
) -> Result<()> {
    let behavior_version = release_info.behavior_version.clone();
    let default_version = format!(
        "{}.{}.{}",
        behavior_version[0],
        behavior_version[1],
        behavior_version[2] + 1
    );
    println!(
        "✨ 预计版本号: {}",
        version.as_deref().unwrap_or(&default_version)
    );
    // 转换版本号
    let version = version
        .as_deref()
        .unwrap_or(&default_version)
        .split(".")
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<u32>, _>>()?;
    let version_value = Value::Array(version.iter().map(|v| Value::from(*v)).collect());
    println!("📦 开始打包, 版本号: {:?}", &version);
    write_to_pack(&project_dir, &version_value)?;
    write_to_manifests(&project_dir, &release_info, &version_value)?;
    // 将 behavior_pack 和 resource_pack 打包到同一个 zip 中
    let output_path = package_folders(&project_dir, &release_info, &default_version)?;
    println!("📦 打包完成: {}", output_path.replace("\\", "/"));
    Ok(())
}

fn package_folders(
    project_dir: &PathBuf,
    release_info: &ReleaseInfo,
    target: &String,
) -> Result<String> {
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
) -> Result<()> {
    if !src_dir.is_dir() {
        return Err(anyhow!("{} is not a directory", src_dir.display()));
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

fn write_to_pack(project_dir: &PathBuf, version: &Value) -> Result<()> {
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
) -> Result<()> {
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

fn write_to_manifest(project_dir: &PathBuf, version: &Value) -> Result<()> {
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
