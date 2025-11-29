use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex::Regex;
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateConfig {
    pub template: TemplateInfo,
    #[serde(default)]
    pub renames: Vec<RenameRule>,
    pub variables: HashMap<String, VariableConfig>,
    pub process: ProcessConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RenameRule {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VariableConfig {
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessConfig {
    pub file_extensions: Vec<String>,
}

pub struct TemplateEngine {
    config: TemplateConfig,
    variables: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn load(template_dir: &Path) -> crate::error::Result<Self> {
        let config_path = template_dir.join("template.toml");
        let content = fs::read_to_string(&config_path)?;
        let config: TemplateConfig = toml::from_str(&content)?;
        
        Ok(Self {
            config,
            variables: HashMap::new(),
        })
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    pub fn validate_variables(&self) -> crate::error::Result<()> {
        for (key, var_config) in &self.config.variables {
            if var_config.required && !self.variables.contains_key(key) {
                return Err(crate::error::CliError::InvalidInput(format!(
                    "缺少必需的变量: {} ({})",
                    key, var_config.description
                )));
            }
        }
        Ok(())
    }

    pub fn process_directory(&self, dir: &Path) -> crate::error::Result<()> {
        self.validate_variables()?;

        self.replace_in_files(dir)?;

        self.apply_renames(dir)?;

        self.verify_no_placeholders(dir)?;

        Ok(())
    }

    fn replace_in_files(&self, dir: &Path) -> crate::error::Result<()> {
        let placeholder_regex = Regex::new(r"\{\{(\w+)\}\}").unwrap();

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if !self.config.process.file_extensions.contains(&ext.to_string()) {
                    continue;
                }
            } else {
                continue;
            }

            let content = fs::read_to_string(path)?;
            let mut updated = content.clone();

            for cap in placeholder_regex.captures_iter(&content) {
                let placeholder = &cap[0];
                let var_name = &cap[1];
                
                if let Some(value) = self.variables.get(var_name) {
                    updated = updated.replace(placeholder, value);
                }
            }

            if updated != content {
                fs::write(path, updated)?;
                println!(" - 处理文件: {}", path.display());
            }
        }

        Ok(())
    }

    fn apply_renames(&self, dir: &Path) -> crate::error::Result<()> {
        for rule in self.config.renames.iter().rev() {
            let from_path = dir.join(&rule.from);
            
            if !from_path.exists() {
                continue;
            }

            let to_path_str = self.replace_placeholders(&rule.to);
            let to_path = dir.join(&to_path_str);

            if let Some(parent) = to_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::rename(&from_path, &to_path)?;
            println!(" - 重命名: {} -> {}", rule.from, to_path_str);
        }

        Ok(())
    }

    fn replace_placeholders(&self, text: &str) -> String {
        let placeholder_regex = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut result = text.to_string();

        for cap in placeholder_regex.captures_iter(text) {
            let placeholder = &cap[0];
            let var_name = &cap[1];
            
            if let Some(value) = self.variables.get(var_name) {
                result = result.replace(placeholder, value);
            }
        }

        result
    }

    fn verify_no_placeholders(&self, dir: &Path) -> crate::error::Result<()> {
        let placeholder_regex = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut found_placeholders = Vec::new();

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if !self.config.process.file_extensions.contains(&ext.to_string()) {
                    continue;
                }
            } else {
                continue;
            }

            let content = fs::read_to_string(path)?;
            
            for cap in placeholder_regex.captures_iter(&content) {
                let var_name = &cap[1];
                found_placeholders.push(format!("{}:{}", path.display(), var_name));
            }
        }

        if !found_placeholders.is_empty() {
            eprintln!("警告: 发现未替换的占位符:");
            for placeholder in found_placeholders {
                eprintln!("  - {}", placeholder);
            }
        }

        Ok(())
    }
}