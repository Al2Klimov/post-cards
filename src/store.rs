use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub domain: String,
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
}

fn dir() -> Option<PathBuf> {
    if let Some(x) = std::env::var_os("XDG_CONFIG_HOME").filter(|x| !x.is_empty()) {
        return Some(PathBuf::from(x).join("post_cards"));
    }
    if let Some(a) = std::env::var_os("APPDATA").filter(|x| !x.is_empty()) {
        return Some(PathBuf::from(a).join("post_cards"));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config").join("post_cards"))
}

fn save<T: Serialize>(name: &str, v: &T) -> Result<(), String> {
    let d = dir().ok_or("cannot determine config directory")?;
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let p = d.join(name);
    let data = serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?;
    std::fs::write(&p, data).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn load<T: for<'a> Deserialize<'a>>(name: &str) -> Option<T> {
    serde_json::from_slice(&std::fs::read(dir()?.join(name)).ok()?).ok()
}

pub fn load_credentials() -> Option<Credentials> {
    load("credentials.json")
}

pub fn save_credentials(c: &Credentials) -> Result<(), String> {
    save("credentials.json", c)
}
