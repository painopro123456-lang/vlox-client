use crust_core::{
    authenticator::{Account, Authenticator},
    foundation::events::Event,
    launcher::{Launch, LaunchOptions, LoaderKind},
};
use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc},
    thread,
};

const MICROSOFT_CLIENT_ID: &str = "09e6a2a2-90ee-47a6-92c4-f8810958de91";
const SERVICE_NAME: &str = "VLOX CLIENT";
const ACCOUNT_KEY: &str = "minecraft-account";
const MINECRAFT_VERSION: &str = "26.2";
const FABRIC_API_FILE: &str = "fabric-api-0.161.0+26.2.jar";
const VLOX_MOD_FILE: &str = "vlox-client-mod-0.1.0.jar";

// These are built/downloaded in the Windows workflow before the Rust executable.
const VLOX_MOD: &[u8] = include_bytes!("../client-mod/build/libs/vlox-client-mod-0.1.0.jar");
const FABRIC_API: &[u8] = include_bytes!("../client-mod/build/libs/fabric-api-0.161.0+26.2.jar");

#[derive(Debug)]
pub enum AuthEvent {
    DeviceCode { code: String, url: String },
    SignedIn { name: String },
    LaunchProgress(String),
    LaunchStarted,
    LaunchExited(String),
    Failed(String),
}

pub fn saved_account_name() -> Option<String> {
    load_session().ok().flatten().map(|(_, name)| name)
}

pub fn start_sign_in(sender: Sender<AuthEvent>) {
    thread::spawn(move || {
        let result = run_async(async {
            let auth = Authenticator::with_client_id(MICROSOFT_CLIENT_ID)
                .map_err(|error| format!("Could not initialize Microsoft sign-in: {error}"))?;
            let flow = auth.device_code().await
                .map_err(|error| format!("Could not request a Microsoft sign-in code: {error}"))?;
            let code = flow.user_code().to_owned();
            let url = flow.verification_uri_complete();
            let _ = sender.send(AuthEvent::DeviceCode { code, url: url.clone() });
            let _ = webbrowser::open(&url);
            let account = flow.wait().await
                .map_err(|error| format!("Microsoft/Minecraft sign-in failed: {error}"))?;
            save_account(&account)?;
            let _ = sender.send(AuthEvent::SignedIn { name: account.name.clone() });
            Ok::<(), String>(())
        });
        if let Err(error) = result {
            let _ = sender.send(AuthEvent::Failed(error));
        }
    });
}

pub fn sign_out() -> Result<(), String> {
    #[cfg(windows)]
    {
        let entry = keyring::Entry::new(SERVICE_NAME, ACCOUNT_KEY)
            .map_err(|error| format!("Could not access Windows Credential Manager: {error}"))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!("Could not remove the saved account: {error}")),
        }
    }
    #[cfg(not(windows))]
    {
        Err("Secure account storage is currently supported on Windows builds.".to_owned())
    }
}

pub fn start_launch(sender: Sender<AuthEvent>) {
    thread::spawn(move || {
        let result = run_async(async {
            let (refresh_token, _) = load_session()?
                .ok_or_else(|| "Sign in with Microsoft before launching Minecraft.".to_owned())?;
            let auth = Authenticator::with_client_id(MICROSOFT_CLIENT_ID)
                .map_err(|error| format!("Could not initialize account refresh: {error}"))?;
            let session = auth.microsoft().refresh(&refresh_token).await
                .map_err(|error| format!("Your Microsoft session needs attention. Sign in again: {error}"))?;
            let account = auth.complete(session).await
                .map_err(|error| format!("Could not refresh your Minecraft profile: {error}"))?;
            save_account(&account)?;

            let minecraft_dir = minecraft_directory()?;
            install_client_mods(&minecraft_dir)?;

            let mut options = LaunchOptions::new(&minecraft_dir, MINECRAFT_VERSION);
            options.loader.kind = Some(LoaderKind::Fabric);
            options.loader.enable = true;
            options.loader.build = "latest".to_owned();
            options.loader.path = "./".to_owned();
            options.download_concurrency = 8;
            options.memory.min = "1G".to_owned();
            options.memory.max = "4G".to_owned();
            options.ignore_log4j = true;

            let progress_sender = sender.clone();
            let events = Arc::new(move |event: Event| {
                match event {
                    Event::Progress { downloaded, total, element } => {
                        let percent = if total > 0 {
                            downloaded.saturating_mul(100) / total
                        } else { 0 };
                        let _ = progress_sender.send(AuthEvent::LaunchProgress(
                            format!("Downloading {element}: {percent}%")
                        ));
                    }
                    Event::Check { checked, total, element } => {
                        let _ = progress_sender.send(AuthEvent::LaunchProgress(
                            format!("{element}: {checked}/{total}")
                        ));
                    }
                    Event::Extract(message) | Event::Patch(message) => {
                        let _ = progress_sender.send(AuthEvent::LaunchProgress(message));
                    }
                    Event::Error(message) => {
                        let _ = progress_sender.send(AuthEvent::LaunchProgress(message));
                    }
                    Event::Speed(_) | Event::Estimated(_) => {}
                }
            });

            let launch = Launch::new(options, account)
                .map_err(|error| format!("Could not prepare Minecraft: {error}"))?
                .with_events(events);
            let _ = sender.send(AuthEvent::LaunchProgress(
                "Preparing Minecraft Java 26.2 + Fabric + VLOX…".to_owned()
            ));
            let mut child = launch.run().await
                .map_err(|error| format!("Minecraft installation or launch failed: {error}"))?;
            let _ = sender.send(AuthEvent::LaunchStarted);
            let status = child.wait().await
                .map_err(|error| format!("Could not wait for Minecraft: {error}"))?;
            let _ = sender.send(AuthEvent::LaunchExited(format!("Minecraft closed with {status}")));
            Ok::<(), String>(())
        });
        if let Err(error) = result {
            let _ = sender.send(AuthEvent::Failed(error));
        }
    });
}

fn run_async<F>(future: F) -> Result<(), String>
where
    F: std::future::Future<Output = Result<(), String>>,
{
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("Could not start launcher runtime: {error}"))?;
    runtime.block_on(future)
}

fn minecraft_directory() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let appdata = std::env::var_os("APPDATA")
            .ok_or_else(|| "Windows AppData folder was not found.".to_owned())?;
        Ok(PathBuf::from(appdata).join(".minecraft"))
    }
    #[cfg(not(windows))]
    {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| "Home directory was not found.".to_owned())?;
        Ok(PathBuf::from(home).join(".minecraft"))
    }
}

fn install_client_mods(minecraft_dir: &PathBuf) -> Result<(), String> {
    let mods_dir = minecraft_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)
        .map_err(|error| format!("Could not create Minecraft mods folder: {error}"))?;
    std::fs::write(mods_dir.join(VLOX_MOD_FILE), VLOX_MOD)
        .map_err(|error| format!("Could not install the VLOX mod: {error}"))?;
    std::fs::write(mods_dir.join(FABRIC_API_FILE), FABRIC_API)
        .map_err(|error| format!("Could not install Fabric API: {error}"))?;
    Ok(())
}

fn load_session() -> Result<Option<(String, String)>, String> {
    #[cfg(windows)]
    {
        let entry = keyring::Entry::new(SERVICE_NAME, ACCOUNT_KEY)
            .map_err(|error| format!("Could not access Windows Credential Manager: {error}"))?;
        match entry.get_password() {
            Ok(serialized) => {
                let value: serde_json::Value = serde_json::from_str(&serialized)
                    .map_err(|error| format!("Saved account data is invalid: {error}"))?;
                let token = value.get("refresh_token").and_then(serde_json::Value::as_str)
                    .filter(|token| !token.is_empty())
                    .ok_or_else(|| "Saved Microsoft session has no refresh token. Sign in again.".to_owned())?;
                let name = value.get("name").and_then(serde_json::Value::as_str)
                    .unwrap_or("Microsoft account");
                Ok(Some((token.to_owned(), name.to_owned())))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(format!("Could not read the saved Microsoft account: {error}")),
        }
    }
    #[cfg(not(windows))]
    {
        Ok(None)
    }
}

fn save_account(account: &Account) -> Result<(), String> {
    #[cfg(windows)]
    {
        let refresh_token = account.refresh_token.as_deref()
            .filter(|token| !token.is_empty())
            .ok_or_else(|| "Microsoft did not provide a refresh token; sign-in cannot be saved.".to_owned())?;
        let serialized = serde_json::json!({
            "refresh_token": refresh_token,
            "name": account.name
        }).to_string();
        let entry = keyring::Entry::new(SERVICE_NAME, ACCOUNT_KEY)
            .map_err(|error| format!("Could not access Windows Credential Manager: {error}"))?;
        entry.set_password(&serialized)
            .map_err(|error| format!("Could not securely save the Microsoft session: {error}"))
    }
    #[cfg(not(windows))]
    {
        let _ = account;
        Err("Secure account storage is currently supported on Windows builds.".to_owned())
    }
}
    #[cfg(windows)]
    {
        let serialized = serde_json::to_string(account)
            .map_err(|error| format!("Could not encode the account session: {error}"))?;
        let entry = keyring::Entry::new(SERVICE_NAME, ACCOUNT_KEY)
            .map_err(|error| format!("Could not access Windows Credential Manager: {error}"))?;
        entry.set_password(&serialized)
            .map_err(|error| format!("Could not securely save the Microsoft session: {error}"))
    }
    #[cfg(not(windows))]
    {
        let _ = account;
        Err("Secure account storage is currently supported on Windows builds.".to_owned())
    }
}
