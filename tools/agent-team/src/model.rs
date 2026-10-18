use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub type Result<T> = std::result::Result<T, String>;
pub const MAX_FRAME: u64 = 262_144;
pub const MAX_RESULT: usize = 24_000;
pub trait Stream: Read + Write + Send {}
impl<T: Read + Write + Send> Stream for T {}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
pub fn id() -> String {
    let mut b = [0u8; 16];
    fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut b))
        .expect("OS random source");
    b.iter().map(|x| format!("{x:02x}")).collect()
}
pub fn clipped(s: &str, max: usize) -> String {
    let mut end = s.len().min(max);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end]
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}
pub fn directory() -> PathBuf {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".local/state")
        });
    base.join("omarchy/agent-team")
}
pub fn runtime() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(directory)
        .join("omarchy-agent-team")
}
pub fn private_dir(p: &std::path::Path) -> Result<()> {
    fs::create_dir_all(p).map_err(|e| e.to_string())?;
    let m = fs::symlink_metadata(p).map_err(|e| e.to_string())?;
    use std::os::unix::fs::MetadataExt;
    if !m.is_dir() || m.uid() != unsafe { libc::geteuid() } {
        return Err("State directory is not owned by this user".into());
    }
    fs::set_permissions(p, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())
}
pub fn atomic_write(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension(format!("{}.tmp", id()));
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&tmp)
            .map_err(|e| e.to_string())?;
        f.write_all(bytes)
            .and_then(|_| f.sync_all())
            .map_err(|e| e.to_string())?;
        fs::rename(&tmp, path).map_err(|e| e.to_string())?;
        fs::File::open(path.parent().unwrap())
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: String,
    pub parent: Option<String>,
    pub provider: String,
    pub model: Option<String>,
    pub cwd: String,
    pub account_home: Option<String>,
    #[serde(default)]
    pub account_homes: std::collections::HashMap<String, Option<String>>,
    pub prompt: String,
    pub status: String,
    pub activity: String,
    pub result: String,
    pub error: Option<String>,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub request_key: Option<String>,
    #[serde(default)]
    pub output_truncated: bool,
}
impl Task {
    pub fn active(&self) -> bool {
        matches!(
            self.status.as_str(),
            "queued" | "running" | "waiting_for_children"
        )
    }
    pub fn public(&self, detail: bool) -> Value {
        json!({"id":self.id,"parent":self.parent,"provider":self.provider,"model":self.model,
            "cwd":self.cwd,"account": if self.account_home.is_some(){"Selected named account"}else{"Main / CLI default"},
            "prompt":clipped(&self.prompt, if detail {12_000}else{180}),"status":self.status,
            "activity":self.activity,"result":if detail {self.result.clone()}else{clipped(&self.result,240)},
            "error":self.error,"created_at":self.created_at,"completed_at":self.completed_at,"output_truncated":self.output_truncated})
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub op: String,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub capability: Option<String>,
}
pub fn required<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("Missing {key}"))
}
