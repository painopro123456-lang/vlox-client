use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

fn appdata() -> Result<PathBuf, String> {
    let root = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .ok_or_else(|| "Could not locate your user data folder.".to_owned())?;
    Ok(root)
}

pub fn minecraft_dir() -> Result<PathBuf, String> {
    Ok(appdata()?.join(".minecraft"))
}

pub fn mods_dir() -> Result<PathBuf, String> {
    Ok(minecraft_dir()?.join("mods"))
}

pub fn list_mods() -> Result<Vec<String>, String> {
    let dir = mods_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("Could not open mods folder: {e}"))?;
    let mut mods = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("Could not read mods folder: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("jar")) {
            mods.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    mods.sort_by_key(|s| s.to_lowercase());
    Ok(mods)
}

pub fn add_mod() -> Result<String, String> {
    let selected = rfd::FileDialog::new()
        .add_filter("Minecraft mod", &["jar"])
        .pick_file()
        .ok_or_else(|| "No mod selected.".to_owned())?;
    let name = selected.file_name().ok_or("Invalid mod filename")?;
    let dir = mods_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join(name);
    fs::copy(&selected, &target).map_err(|e| format!("Could not copy mod: {e}"))?;
    Ok(format!("Added {}. Restart Minecraft to load it.", name.to_string_lossy()))
}

pub fn remove_mod(name: &str) -> Result<(), String> {
    let safe = Path::new(name);
    if safe.components().count() != 1 || !name.to_lowercase().ends_with(".jar") {
        return Err("Invalid mod filename.".to_owned());
    }
    fs::remove_file(mods_dir()?.join(safe)).map_err(|e| format!("Could not remove mod: {e}"))
}

pub fn open_mods_folder() -> Result<(), String> {
    let dir = mods_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    std::process::Command::new("explorer").arg(&dir).spawn().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&dir).spawn().map_err(|e| e.to_string())?;
    #[cfg(all(not(windows), not(target_os = "macos")))]
    std::process::Command::new("xdg-open").arg(&dir).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

fn instances_root() -> Result<PathBuf, String> {
    let dir = appdata()?.join(".vlox").join("instances");
    fs::create_dir_all(&dir).map_err(|e| format!("Could not create VLOX instances folder: {e}"))?;
    Ok(dir)
}

pub fn list_instances() -> Result<Vec<String>, String> {
    let root = instances_root()?;
    let mut names = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_dir() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    names.sort_by_key(|s| s.to_lowercase());
    Ok(names)
}

pub fn instance_path(name: &str) -> Result<PathBuf, String> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err("Invalid modpack name.".to_owned());
    }
    Ok(instances_root()?.join(name))
}

pub fn delete_instance(name: &str) -> Result<(), String> {
    let path = instance_path(name)?;
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| format!("Could not delete modpack instance: {e}"))?;
    }
    Ok(())
}

fn safe_relative(path: &str) -> Result<PathBuf, String> {
    let p = Path::new(path);
    if p.as_os_str().is_empty() || p.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(format!("Unsafe path in modpack: {path}"));
    }
    Ok(p.to_path_buf())
}

pub fn import_modpack() -> Result<String, String> {
    let archive_path = rfd::FileDialog::new()
        .add_filter("Modrinth modpack", &["mrpack"])
        .pick_file()
        .ok_or_else(|| "No modpack selected.".to_owned())?;
    let file = fs::File::open(&archive_path).map_err(|e| format!("Could not open modpack: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid .mrpack archive: {e}"))?;
    let mut index_text = String::new();
    archive.by_name("modrinth.index.json")
        .map_err(|_| "This is not a valid Modrinth .mrpack (index missing).".to_owned())?
        .read_to_string(&mut index_text).map_err(|e| e.to_string())?;
    let index: serde_json::Value = serde_json::from_str(&index_text)
        .map_err(|e| format!("Invalid modpack index: {e}"))?;
    let game_version = index.pointer("/dependencies/minecraft").and_then(|v| v.as_str()).unwrap_or("");
    if game_version != "26.2" {
        return Err(format!("This VLOX build supports Minecraft 26.2 modpacks. This pack targets {game_version}."));
    }
    let fabric = index.pointer("/dependencies/fabric-loader").and_then(|v| v.as_str());
    if fabric.is_none() {
        return Err("This modpack does not declare Fabric Loader. VLOX currently supports Fabric modpacks.".to_owned());
    }
    let raw_name = archive_path.file_stem().and_then(|s| s.to_str()).unwrap_or("Imported Modpack");
    let safe_name: String = raw_name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }).collect();
    let target = instance_path(&safe_name)?;
    if target.exists() {
        return Err(format!("A modpack named '{safe_name}' is already installed. Delete it first or rename the .mrpack file."));
    }
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;

    let install_result = (|| -> Result<(), String> {
        let files = index.get("files").and_then(|v| v.as_array()).ok_or("Modpack file list is missing")?;
        let client = reqwest::blocking::Client::builder().user_agent("VLOX-CLIENT/0.4").build()
            .map_err(|e| format!("Could not initialize downloader: {e}"))?;
        for item in files {
            if item.pointer("/env/client").and_then(|v| v.as_str()) == Some("unsupported") {
                continue;
            }
            let rel = item.get("path").and_then(|v| v.as_str()).ok_or("Modpack contains a file without a path")?;
            let rel_path = safe_relative(rel)?;
            let urls = item.get("downloads").and_then(|v| v.as_array()).ok_or("Modpack file has no download URL")?;
            let url = urls.first().and_then(|v| v.as_str()).ok_or("Modpack file has an invalid download URL")?;
            let response = client.get(url).send().map_err(|e| format!("Download failed for {rel}: {e}"))?
                .error_for_status().map_err(|e| format!("Download failed for {rel}: {e}"))?;
            let bytes = response.bytes().map_err(|e| e.to_string())?;
            if let Some(expected) = item.get("fileSize").and_then(|v| v.as_u64()) {
                if bytes.len() as u64 != expected {
                    return Err(format!("File size check failed for {rel}."));
                }
            }
            let out = target.join(rel_path);
            if let Some(parent) = out.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
            let mut output = fs::File::create(out).map_err(|e| e.to_string())?;
            output.write_all(&bytes).map_err(|e| e.to_string())?;
        }

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_owned();
            let Some(rest) = name.strip_prefix("overrides/") else { continue };
            if rest.is_empty() { continue; }
            let rel = safe_relative(rest)?;
            let out = target.join(rel);
            if entry.is_dir() {
                fs::create_dir_all(out).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
                let mut output = fs::File::create(out).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut output).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    })();

    if let Err(error) = install_result {
        let _ = fs::remove_dir_all(&target);
        return Err(error);
    }
    Ok(format!("Installed '{safe_name}' for Minecraft 26.2. Select it and launch from VLOX."))
}
