use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

include!(concat!(env!("OUT_DIR"), "/curseforge_key.rs"));

fn curseforge_api_key() -> Result<String, String> {
    if ENCRYPTED_CURSEFORGE_KEY.is_empty() {
        return Err("CurseForge is not configured in this build. Add the CURSEFORGE_API_KEY repository secret and rebuild.".to_owned());
    }
    Ok(ENCRYPTED_CURSEFORGE_KEY.iter().map(|byte| (byte ^ 0xA7) as char).collect())
}

fn curseforge_get(client: &reqwest::blocking::Client, url: String) -> Result<reqwest::blocking::RequestBuilder, String> {
    let key = curseforge_api_key()?;
    Ok(client.get(url).header("x-api-key", key))
}

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
    install_mrpack_file(&archive_path)
}

fn install_mrpack_file(archive_path: &Path) -> Result<String, String> {
    let file = fs::File::open(archive_path).map_err(|e| format!("Could not open modpack: {e}"))?;
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


#[derive(Clone, Debug)]
pub struct ModrinthPack {
    pub title: String,
    pub slug: String,
    pub description: String,
    pub downloads: u64,
}

pub fn search_modrinth_modpacks(query: &str) -> Result<Vec<ModrinthPack>, String> {
    let client = reqwest::blocking::Client::builder().user_agent("VLOX-CLIENT/0.4").build().map_err(|e| e.to_string())?;
    let response = client.get("https://api.modrinth.com/v2/search")
        .query(&[("query", query), ("limit", "20"), ("facets", "[[\"project_type:modpack\"],[\"versions:26.2\"],[\"categories:fabric\"]]")])
        .send().map_err(|e| format!("Modrinth search failed: {e}"))?
        .error_for_status().map_err(|e| format!("Modrinth search failed: {e}"))?;
    let value: serde_json::Value = response.json().map_err(|e| e.to_string())?;
    let hits = value.get("hits").and_then(|v| v.as_array()).ok_or("Modrinth returned an invalid search response")?;
    Ok(hits.iter().filter_map(|hit| {
        Some(ModrinthPack {
            title: hit.get("title")?.as_str()?.to_owned(),
            slug: hit.get("slug")?.as_str()?.to_owned(),
            description: hit.get("description").and_then(|v| v.as_str()).unwrap_or("").to_owned(),
            downloads: hit.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    }).collect())
}

pub fn install_modrinth_pack(slug: &str) -> Result<String, String> {
    if slug.is_empty() || !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("Invalid Modrinth project slug.".to_owned());
    }
    let client = reqwest::blocking::Client::builder().user_agent("VLOX-CLIENT/0.4").build().map_err(|e| e.to_string())?;
    let versions_url = format!("https://api.modrinth.com/v2/project/{slug}/version");
    let response = client.get(versions_url).query(&[("game_versions", "[\"26.2\"]"), ("loaders", "[\"fabric\"]")])
        .send().map_err(|e| format!("Could not find Modrinth pack versions: {e}"))?
        .error_for_status().map_err(|e| format!("Could not find Modrinth pack versions: {e}"))?;
    let versions: serde_json::Value = response.json().map_err(|e| e.to_string())?;
    let version = versions.as_array().and_then(|a| a.first()).ok_or("No Fabric 26.2 version is available for this Modrinth pack.")?;
    let files = version.get("files").and_then(|v| v.as_array()).ok_or("Modrinth version has no files")?;
    let file = files.iter().find(|f| f.get("primary").and_then(|v| v.as_bool()) == Some(true)).or_else(|| files.first()).ok_or("Modrinth version has no downloadable file")?;
    let url = file.get("url").and_then(|v| v.as_str()).ok_or("Modrinth file has no download URL")?;
    let filename = file.get("filename").and_then(|v| v.as_str()).unwrap_or("download.mrpack");
    if !filename.to_lowercase().ends_with(".mrpack") { return Err("Modrinth project version did not provide an .mrpack file.".to_owned()); }
    let bytes = client.get(url).send().map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?.bytes().map_err(|e| e.to_string())?;
    let temp = appdata()?.join(".vlox").join(format!("download-{slug}.mrpack"));
    if let Some(parent) = temp.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    fs::write(&temp, &bytes).map_err(|e| e.to_string())?;
    let result = install_mrpack_file(&temp);
    let _ = fs::remove_file(&temp);
    result
}

pub fn import_curseforge_modpack() -> Result<String, String> {
    let archive_path = rfd::FileDialog::new().add_filter("CurseForge modpack export", &["zip"]).pick_file()
        .ok_or_else(|| "No CurseForge modpack selected.".to_owned())?;
    import_curseforge_archive(&archive_path)
}

fn import_curseforge_archive(archive_path: &Path) -> Result<String, String> {
    let file = fs::File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid CurseForge archive: {e}"))?;
    let mut manifest_text = String::new();
    archive.by_name("manifest.json").map_err(|_| "CurseForge export is missing manifest.json.".to_owned())?
        .read_to_string(&mut manifest_text).map_err(|e| e.to_string())?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_text).map_err(|e| e.to_string())?;
    let mc = manifest.pointer("/minecraft/version").and_then(|v| v.as_str()).unwrap_or("");
    if mc != "26.2" { return Err(format!("This VLOX build supports Minecraft 26.2 modpacks. This pack targets {mc}.")); }
    let loaders = manifest.pointer("/minecraft/modLoaders").and_then(|v| v.as_array()).ok_or("CurseForge manifest has no mod loader")?;
    if !loaders.iter().any(|v| v.get("id").and_then(|x| x.as_str()).is_some_and(|id| id.starts_with("fabric-"))) {
        return Err("This CurseForge pack is not a Fabric modpack.".to_owned());
    }
    let raw_name = manifest.get("name").and_then(|v| v.as_str()).or_else(|| archive_path.file_stem().and_then(|v| v.to_str())).unwrap_or("CurseForge Pack");
    let safe_name: String = raw_name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }).collect();
    let target = instance_path(&safe_name)?;
    if target.exists() { return Err(format!("A modpack named '{safe_name}' is already installed.")); }
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        let client = reqwest::blocking::Client::builder().user_agent("VLOX-CLIENT/0.4").build().map_err(|e| e.to_string())?;
        let files = manifest.get("files").and_then(|v| v.as_array()).ok_or("CurseForge file list is missing")?;
        for item in files {
            let project = item.get("projectID").and_then(|v| v.as_u64()).ok_or("Invalid CurseForge project ID")?;
            let file_id = item.get("fileID").and_then(|v| v.as_u64()).ok_or("Invalid CurseForge file ID")?;
            let url = format!("{CURSEFORGE_PROXY_BASE}/v1/mods/{project}/files/{file_id}");
            let data: serde_json::Value = curseforge_get(&client, url)?.send()
                .map_err(|e| format!("CurseForge API request failed: {e}"))?.error_for_status()
                .map_err(|e| format!("CurseForge API rejected the key or file request: {e}"))?.json().map_err(|e| e.to_string())?;
            let download_url = data.pointer("/data/downloadUrl").and_then(|v| v.as_str())
                .ok_or("CurseForge did not provide a download URL for one of the mod files")?;
            let filename = data.pointer("/data/fileName").and_then(|v| v.as_str()).ok_or("CurseForge file has no filename")?;
            let bytes = client.get(download_url).send().map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?.bytes().map_err(|e| e.to_string())?;
            let out = target.join("mods").join(safe_relative(filename)?);
            if let Some(parent) = out.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
            fs::write(out, &bytes).map_err(|e| e.to_string())?;
        }
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_owned();
            let Some(rest) = name.strip_prefix("overrides/") else { continue };
            if rest.is_empty() { continue; }
            let out = target.join(safe_relative(rest)?);
            if entry.is_dir() { fs::create_dir_all(out).map_err(|e| e.to_string())?; }
            else {
                if let Some(parent) = out.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
                let mut output = fs::File::create(out).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut output).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    })();
    if let Err(error) = result { let _ = fs::remove_dir_all(&target); return Err(error); }
    Ok(format!("Installed CurseForge pack '{safe_name}'. Select it and launch from VLOX."))
}


#[derive(Clone, Debug)]
pub struct CurseForgePack {
    pub id: u64,
    pub title: String,
    pub summary: String,
    pub downloads: u64,
}

pub fn search_curseforge_modpacks(query: &str) -> Result<Vec<CurseForgePack>, String> {
    let client = reqwest::blocking::Client::builder().user_agent("VLOX-CLIENT/0.4").build().map_err(|e| e.to_string())?;
    let response = curseforge_get(&client, "https://api.curseforge.com/v1/mods/search".to_owned())?
        .query(&[
            ("gameId", "432"), ("classId", "4471"), ("searchFilter", query),
            ("pageSize", "20"), ("sortField", "6"), ("sortOrder", "desc"),
            ("gameVersion", "26.2"), ("modLoaderType", "4"),
        ])
        .send().map_err(|e| format!("CurseForge search failed: {e}"))?
        .error_for_status().map_err(|e| format!("CurseForge rejected the API key or search: {e}"))?;
    let value: serde_json::Value = response.json().map_err(|e| e.to_string())?;
    let data = value.get("data").and_then(|v| v.as_array()).ok_or("CurseForge returned an invalid search response")?;
    Ok(data.iter().filter_map(|item| Some(CurseForgePack {
        id: item.get("id")?.as_u64()?,
        title: item.get("name")?.as_str()?.to_owned(),
        summary: item.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_owned(),
        downloads: item.get("downloadCount").and_then(|v| v.as_u64()).unwrap_or(0),
    })).collect())
}

pub fn install_curseforge_project(project_id: u64) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder().user_agent("VLOX-CLIENT/0.4").build().map_err(|e| e.to_string())?;
    let response = curseforge_get(&client, format!("https://api.curseforge.com/v1/mods/{project_id}/files"))?
        .query(&[("gameVersion", "26.2"), ("modLoaderType", "4"), ("pageSize", "20"), ("index", "0")])
        .send().map_err(|e| e.to_string())?.error_for_status().map_err(|e| format!("CurseForge file lookup failed: {e}"))?;
    let value: serde_json::Value = response.json().map_err(|e| e.to_string())?;
    let files = value.get("data").and_then(|v| v.as_array()).ok_or("CurseForge returned no files")?;
    let pack_file = files.iter().find(|f| f.get("isAvailable").and_then(|v| v.as_bool()) != Some(false)
        && f.get("fileName").and_then(|v| v.as_str()).is_some_and(|n| n.to_lowercase().ends_with(".zip")))
        .ok_or("No compatible CurseForge modpack ZIP was found for Minecraft 26.2 / Fabric.")?;
    let url = pack_file.get("downloadUrl").and_then(|v| v.as_str())
        .ok_or("CurseForge did not provide a direct download URL for this file")?;
    let filename = pack_file.get("fileName").and_then(|v| v.as_str()).unwrap_or("curseforge-pack.zip");
    let bytes = client.get(url).send().map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?.bytes().map_err(|e| e.to_string())?;
    let temp = appdata()?.join(".vlox").join(format!("curseforge-{project_id}.zip"));
    if let Some(parent) = temp.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    fs::write(&temp, &bytes).map_err(|e| e.to_string())?;
    let result = import_curseforge_archive(&temp);
    let _ = fs::remove_file(&temp);
    result.map(|message| format!("{message} ({filename})"))
}
