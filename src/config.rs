use crate::model::{AboutList, HomeProfile, ProjectList, TlsConfig};
use serde::Deserialize;
use std::fs;

pub fn load_profile() -> HomeProfile {
    let profile_str = match fs::read_to_string("config.toml") {
        Ok(s) => s,
        Err(err) => {
            eprintln!("警告：读取 config.toml 失败 ({}), 使用默认配置", err);
            return HomeProfile::default();
        }
    };
    match toml::from_str(&profile_str) {
        Ok(profile) => profile,
        Err(err) => {
            eprintln!("警告: 解析 config.toml 失败 ({})，使用默认配置", err);
            HomeProfile::default()
        }
    }
}

pub fn load_projects() -> ProjectList {
    // 尝试读取 projects.toml
    let content = match std::fs::read_to_string("projects.toml") {
        Ok(s) => s,
        Err(_) => return ProjectList::default(), // 找不到文件，直接给默认值
    };

    // 尝试解析 TOML 内容
    match toml::from_str::<ProjectList>(&content) {
        Ok(list) => list,
        Err(e) => {
            eprintln!("解析 projects.toml 失败: {}, 使用默认配置", e);
            ProjectList::default()
        }
    }
}

pub fn load_about_items() -> AboutList {
    // 尝试读取 config.toml
    let content = match std::fs::read_to_string("config.toml") {
        Ok(s) => s,
        Err(_) => return AboutList::default(), // 找不到文件，直接给默认值
    };

    // 尝试解析 TOML 内容
    match toml::from_str::<AboutList>(&content) {
        Ok(list) => list,
        Err(e) => {
            eprintln!("解析 config.toml 失败: {}, 使用默认配置", e);
            AboutList::default()
        }
    }
}

pub fn load_tls_config() -> Option<TlsConfig> {
    let content = fs::read_to_string("config.toml").ok()?;
    // 将整个文件解析为 toml::Value，以便提取 [tls] 节
    let value: toml::Value = toml::from_str(&content).ok()?;
    let tls_value = value.get("tls")?;
    // 将 tls 节反序列化为 TlsConfig
    TlsConfig::deserialize(tls_value.clone()).ok()
}
