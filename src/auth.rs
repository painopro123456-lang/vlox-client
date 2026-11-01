use reqwest::blocking::{Client, Response};
use serde_json::{json, Value};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

const MICROSOFT_CLIENT_ID: &str = "09e6a2a2-90ee-47a6-92c4-f8810958de91";
const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

#[derive(Debug)]
pub enum AuthEvent {
    DeviceCode { user_code: String, verification_uri: String },
    SignedIn { name: String, uuid: String },
    Failed(String),
}

pub fn start_sign_in(sender: Sender<AuthEvent>) {
    thread::spawn(move || {
        if let Err(error) = authenticate(&sender) {
            let _ = sender.send(AuthEvent::Failed(error));
        }
    });
}

fn authenticate(sender: &Sender<AuthEvent>) -> Result<(), String> {
    let client = Client::builder()
        .user_agent("VLOX-CLIENT/0.1.0")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("Could not start Microsoft sign-in: {error}"))?;

    let device_response = client.post(DEVICE_CODE_URL)
        .form(&[("client_id", MICROSOFT_CLIENT_ID), ("scope", "XboxLive.signin offline_access")])
        .send().map_err(|error| format!("Could not contact Microsoft: {error}"))?;
    let device = response_json(device_response, "Microsoft device-code request")?;
    let device_code = string_field(&device, "device_code")?;
    let user_code = string_field(&device, "user_code")?;
    let verification_uri = device.get("verification_uri").and_then(Value::as_str)
        .unwrap_or("https://microsoft.com/devicelogin").to_owned();
    let expires_in = device.get("expires_in").and_then(Value::as_u64).unwrap_or(900);
    let mut interval = device.get("interval").and_then(Value::as_u64).unwrap_or(5);

    let _ = sender.send(AuthEvent::DeviceCode { user_code, verification_uri: verification_uri.clone() });
    let _ = webbrowser::open(&verification_uri);

    let started = Instant::now();
    let microsoft_access_token = loop {
        if started.elapsed().as_secs() >= expires_in {
            return Err("Microsoft sign-in code expired. Please try again.".to_owned());
        }
        thread::sleep(Duration::from_secs(interval));
        let response = client.post(TOKEN_URL).form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", MICROSOFT_CLIENT_ID),
            ("device_code", device_code.as_str()),
        ]).send().map_err(|error| format!("Microsoft sign-in polling failed: {error}"))?;
        let token: Value = response.json()
            .map_err(|error| format!("Microsoft returned an unreadable token response: {error}"))?;
        if let Some(access_token) = token.get("access_token").and_then(Value::as_str) {
            break access_token.to_owned();
        }
        match token.get("error").and_then(Value::as_str).unwrap_or("") {
            "authorization_pending" => continue,
            "slow_down" => { interval += 5; continue; }
            "authorization_declined" => return Err("Microsoft sign-in was declined.".to_owned()),
            "expired_token" => return Err("Microsoft sign-in code expired. Please try again.".to_owned()),
            "bad_verification_code" => return Err("Microsoft rejected the sign-in code. Please try again.".to_owned()),
            other => {
                let detail = token.get("error_description").and_then(Value::as_str).unwrap_or(other);
                return Err(format!("Microsoft sign-in failed: {detail}"));
            }
        }
    };

    let xbox_response = client.post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&json!({
            "Properties": {"AuthMethod":"RPS","SiteName":"user.auth.xboxlive.com","RpsTicket":format!("d={microsoft_access_token}")},
            "RelyingParty":"http://auth.xboxlive.com","TokenType":"JWT"
        })).send().map_err(|error| format!("Xbox authentication request failed: {error}"))?;
    let xbox = response_json(xbox_response, "Xbox Live authentication")?;
    let xbox_token = string_field(&xbox, "Token")?;
    let user_hash = xbox.pointer("/DisplayClaims/xui/0/uhs").and_then(Value::as_str)
        .ok_or_else(|| "Xbox Live did not return an account identifier.".to_owned())?;

    let xsts_response = client.post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&json!({
            "Properties":{"SandboxId":"RETAIL","UserTokens":[xbox_token]},
            "RelyingParty":"rp://api.minecraftservices.com/","TokenType":"JWT"
        })).send().map_err(|error| format!("Xbox security-token request failed: {error}"))?;
    let xsts = response_json(xsts_response, "Xbox security-token authorization")?;
    let xsts_token = string_field(&xsts, "Token")?;

    let minecraft_response = client.post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&json!({"identityToken":format!("XBL3.0 x={user_hash};{xsts_token}")}))
        .send().map_err(|error| format!("Minecraft account exchange failed: {error}"))?;
    let minecraft = response_json(minecraft_response, "Minecraft Services login")?;
    let minecraft_token = string_field(&minecraft, "access_token")?;

    let profile_response = client.get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(minecraft_token).send()
        .map_err(|error| format!("Minecraft profile request failed: {error}"))?;
    let profile = response_json(profile_response, "Minecraft profile verification")?;
    let (name, uuid) = parse_profile(&profile)?;
    let _ = sender.send(AuthEvent::SignedIn { name, uuid });
    Ok(())
}

fn response_json(response: Response, stage: &str) -> Result<Value, String> {
    let status = response.status();
    let body = response.text().map_err(|error| format!("{stage} response could not be read: {error}"))?;
    if !status.is_success() {
        let detail = serde_json::from_str::<Value>(&body).ok().and_then(|value| {
            value.get("errorMessage").or_else(|| value.get("Message"))
                .or_else(|| value.get("error_description")).and_then(Value::as_str).map(str::to_owned)
        }).unwrap_or_else(|| format!("HTTP {status}"));
        return Err(format!("{stage} failed: {detail}"));
    }
    serde_json::from_str(&body).map_err(|error| format!("{stage} returned invalid JSON: {error}"))
}

fn string_field(value: &Value, field: &str) -> Result<String, String> {
    value.get(field).and_then(Value::as_str).map(str::to_owned)
        .ok_or_else(|| format!("Authentication response is missing {field}."))
}

fn parse_profile(profile: &Value) -> Result<(String, String), String> {
    let name = string_field(profile, "name")?;
    let uuid = string_field(profile, "id")?;
    if name.trim().is_empty() || uuid.trim().is_empty() {
        return Err("Minecraft returned an empty profile.".to_owned());
    }
    Ok((name, uuid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minecraft_profile_is_parsed() {
        let profile = json!({"id":"0123456789abcdef0123456789abcdef","name":"VloxPlayer"});
        let parsed = parse_profile(&profile).expect("valid profile");
        assert_eq!(parsed.0, "VloxPlayer");
        assert_eq!(parsed.1, "0123456789abcdef0123456789abcdef");
    }

    #[test]
    fn missing_minecraft_profile_is_rejected() {
        let profile = json!({"id":"0123456789abcdef0123456789abcdef"});
        assert!(parse_profile(&profile).is_err());
    }
}
