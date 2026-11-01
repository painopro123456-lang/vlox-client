use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=VLOX_CURSEFORGE_API_KEY");
    let key = env::var("VLOX_CURSEFORGE_API_KEY").unwrap_or_default();
    let encrypted: Vec<u8> = key.bytes().map(|byte| byte ^ 0xA7).collect();
    let values = encrypted.iter().map(u8::to_string).collect::<Vec<_>>().join(",");
    let generated = format!("const ENCRYPTED_CURSEFORGE_KEY: &[u8] = &[{values}];");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set")).join("curseforge_key.rs");
    fs::write(out, generated).expect("write encrypted CurseForge key");
}
