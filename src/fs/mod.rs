use std::fs;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;
use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bookmarks: Vec<String>,
    pub last_dir: Option<String>,
    pub openers: Vec<Opener>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Opener {
    pub name: String,
    pub extensions: Vec<String>,
    pub command: String,
    pub args: Vec<String>,
}

pub fn list_dir(path: &Path) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        entries.push(FileEntry {
            path: entry.path(),
            is_dir: file_type.is_dir(),
        });
    }
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let a_name = a
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_ascii_lowercase();
                let b_name = b
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_ascii_lowercase();
                a_name.cmp(&b_name)
            }
        }
    });
    Ok(entries)
}

pub fn create_file(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::File::create(path)?;
    Ok(())
}

pub fn create_dir(path: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn delete_entry(path: &Path) -> anyhow::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn rename_entry(source: &Path, dest: &Path) -> anyhow::Result<()> {
    fs::rename(source, dest)?;
    Ok(())
}

pub fn move_entry(source: &Path, dest: &Path) -> anyhow::Result<()> {
    fs::rename(source, dest)?;
    Ok(())
}

pub fn copy_entry(source: &Path, dest: &Path) -> anyhow::Result<()> {
    if source.is_dir() {
        copy_dir_recursive(source, dest)?;
    } else {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, dest)?;
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in WalkDir::new(source) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(source)?;
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

pub fn load_config() -> Config {
    let path = config_path();
    let Ok(contents) = fs::read_to_string(&path) else {
        let config = default_config();
        let _ = save_config(&config);
        return config;
    };
    serde_json::from_str(&contents).unwrap_or_else(|_| {
        let config = default_config();
        let _ = save_config(&config);
        config
    })
}

pub fn save_config(config: &Config) -> anyhow::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

fn config_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("chatak").join("config.json");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("chatak")
        .join("config.json")
}

fn default_config() -> Config {
    Config {
        bookmarks: Vec::new(),
        last_dir: None,
        openers: vec![
            Opener {
                name: "pdf".to_string(),
                extensions: vec!["pdf".to_string()],
                command: "zathura".to_string(),
                args: vec!["{path}".to_string()],
            },
            Opener {
                name: "images".to_string(),
                extensions: vec![
                    "png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                command: "feh".to_string(),
                args: vec!["{path}".to_string()],
            },
            Opener {
                name: "text".to_string(),
                extensions: vec![
                    "txt", "md", "rs", "toml", "yaml", "yml", "json", "js", "ts", "jsx", "tsx",
                    "py", "go", "c", "h", "cpp", "hpp", "java", "kt", "kts", "cs", "html", "css",
                    "scss", "sql", "sh", "bash", "zsh", "fish", "ini", "conf", "cfg", "env",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                command: "nvim".to_string(),
                args: vec!["{path}".to_string()],
            },
        ],
    }
}

pub fn build_preview(path: &Path, max_bytes: usize, max_lines: usize) -> anyhow::Result<String> {
    let meta = fs::metadata(path)?;
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if meta.is_dir() {
        let count = fs::read_dir(path)?.count();
        return Ok(format!("Directory\n{}\nEntries: {}", name, count));
    }
    let modified = meta
        .modified()
        .ok()
        .map(|t| DateTime::<Local>::from(t).format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let size = meta.len();
    let snippet = read_text_snippet(path, max_bytes, max_lines)?;
    Ok(format!(
        "File\n{}\nSize: {}\nModified: {}\n\n{}",
        name, size, modified, snippet
    ))
}

fn read_text_snippet(path: &Path, max_bytes: usize, max_lines: usize) -> anyhow::Result<String> {
    let bytes = fs::read(path)?;
    let slice = if bytes.len() > max_bytes {
        &bytes[..max_bytes]
    } else {
        bytes.as_slice()
    };
    let Ok(text) = String::from_utf8(slice.to_vec()) else {
        return Ok("[binary file]".to_string());
    };
    let mut lines = Vec::new();
    for line in text.lines().take(max_lines) {
        lines.push(line);
    }
    if lines.is_empty() {
        Ok("[empty file]".to_string())
    } else {
        Ok(lines.join("\n"))
    }
}
