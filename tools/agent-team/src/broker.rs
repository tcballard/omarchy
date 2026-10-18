use crate::{model::*, runner};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read},
    net::TcpListener,
    os::unix::{
        fs::{OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
        net::UnixListener,
    },
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub struct State {
    pub tasks: Vec<Task>,
    pub cancel: HashMap<String, Arc<AtomicBool>>,
    pub capabilities: HashMap<String, String>,
}
pub type Shared = Arc<Mutex<State>>;
impl State {
    pub fn save(&self) -> Result<()> {
        atomic_write(
            &directory().join("tasks.json"),
            &serde_json::to_vec(&self.tasks).map_err(|e| e.to_string())?,
        )
    }
}

fn string(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}
fn authorize(s: &State, r: &Request) -> Result<()> {
    if let Some(parent) = &r.parent {
        if s.capabilities.get(parent) != r.capability.as_ref() {
            return Err("Invalid or expired task capability".into());
        }
        if !s.tasks.iter().any(|t| &t.id == parent && t.active()) {
            return Err("Parent task is no longer active".into());
        }
        if !matches!(r.op.as_str(), "start" | "read" | "cancel" | "doctor") {
            return Err("Operation unavailable to delegated clients".into());
        }
    } else if r.capability.is_some() {
        return Err("Capability requires a parent task".into());
    }
    Ok(())
}
fn scope_target(s: &State, r: &Request) -> Result<usize> {
    let id = required(&r.params, "taskId")?;
    let i = s
        .tasks
        .iter()
        .position(|t| t.id == id)
        .ok_or("Task not found")?;
    if let Some(parent) = &r.parent
        && s.tasks[i].parent.as_ref() != Some(parent)
    {
        return Err("Task belongs to another parent".into());
    }
    Ok(i)
}
fn cancel_tree(s: &mut State, id: &str) {
    let ids: Vec<String> = s
        .tasks
        .iter()
        .filter(|t| t.id == id || t.parent.as_deref() == Some(id))
        .map(|t| t.id.clone())
        .collect();
    for id in ids {
        if let Some(flag) = s.cancel.get(&id) {
            flag.store(true, Ordering::SeqCst);
        }
        if let Some(t) = s.tasks.iter_mut().find(|t| t.id == id && t.active()) {
            t.status = "cancelled".into();
            t.activity = "Cancellation requested".into();
            t.completed_at = Some(now());
        }
        s.capabilities.remove(&id);
    }
}

pub fn handle(state: &Shared, r: Request) -> Result<Value> {
    {
        let s = state.lock().map_err(|_| "Broker lock poisoned")?;
        authorize(&s, &r)?;
    }
    match r.op.as_str() {
        "doctor" => Ok(runner::doctor()),
        "list" => {
            let s = state.lock().unwrap();
            let tasks: Vec<Value> = s
                .tasks
                .iter()
                .rev()
                .take(40)
                .map(|t| t.public(false))
                .collect();
            let selected = string(&r.params, "selectedId")
                .and_then(|id| s.tasks.iter().find(|t| t.id == id))
                .map(|t| t.public(true));
            Ok(
                json!({"tasks":tasks,"selected":selected,"active":s.tasks.iter().filter(|t|t.active()).count(),"mode":"read-only"}),
            )
        }
        "read" => {
            let wait = r
                .params
                .get("waitMs")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                .min(10_000);
            let started = std::time::Instant::now();
            loop {
                let s = state.lock().unwrap();
                authorize(&s, &r)?;
                let i = scope_target(&s, &r)?;
                if !s.tasks[i].active() || started.elapsed().as_millis() >= wait as u128 {
                    let mut out = s.tasks[i].public(true);
                    out["children"] = json!(
                        s.tasks
                            .iter()
                            .filter(|t| t.parent.as_deref() == Some(&s.tasks[i].id))
                            .map(|t| t.public(false))
                            .collect::<Vec<_>>()
                    );
                    return Ok(out);
                }
                drop(s);
                thread::sleep(Duration::from_millis(100));
            }
        }
        "cancel" => {
            let mut s = state.lock().unwrap();
            authorize(&s, &r)?;
            let i = scope_target(&s, &r)?;
            let id = s.tasks[i].id.clone();
            cancel_tree(&mut s, &id);
            s.save()?;
            Ok(s.tasks[i].public(true))
        }
        "start" => start(state, r),
        _ => Err("Unknown broker operation".into()),
    }
}

fn start(state: &Shared, r: Request) -> Result<Value> {
    let prompt = required(&r.params, "task")?.to_string();
    if prompt.len() > 12_000 {
        return Err("Brief exceeds 12,000 bytes; split it into bounded tasks".into());
    }
    let mut provider = required(&r.params, "provider")?.to_string();
    if provider == "default" {
        provider = runner::default_provider()?;
    }
    if !matches!(provider.as_str(), "codex" | "claude") {
        return Err(format!(
            "{provider} has no execution adapter in this prototype; choose Codex or Claude"
        ));
    }
    let model = string(&r.params, "model");
    if model
        .as_ref()
        .is_some_and(|s| s.len() > 100 || s.chars().any(char::is_control))
    {
        return Err("Invalid model".into());
    }
    let mut s = state.lock().unwrap();
    authorize(&s, &r)?;
    let request_key = string(&r.params, "clientRequestId");
    if request_key
        .as_ref()
        .is_some_and(|k| k.is_empty() || k.len() > 128)
    {
        return Err("clientRequestId must contain 1–128 bytes".into());
    }
    let account_homes = if let Some(parent) = &r.parent {
        let lead = s
            .tasks
            .iter()
            .find(|t| &t.id == parent)
            .ok_or("Parent not found")?;
        if lead.provider == provider {
            return Err("Choose the other provider for an independent review".into());
        }
        lead.account_homes.clone()
    } else {
        let mut homes = HashMap::new();
        for name in ["codex", "claude"] {
            runner::executable(name)?;
            homes.insert(name.into(), runner::selected_home(name)?);
        }
        homes
    };
    let cwd = if let Some(parent) = &r.parent {
        let p = s
            .tasks
            .iter()
            .find(|t| &t.id == parent)
            .ok_or("Parent not found")?;
        if p.parent.is_some() {
            return Err("Nested delegation is unavailable in this prototype".into());
        }
        if s.tasks
            .iter()
            .filter(|t| t.parent.as_deref() == Some(parent))
            .count()
            >= 32
        {
            return Err("A review is limited to 32 delegated tasks".into());
        }
        p.cwd.clone()
    } else {
        let p = std::path::PathBuf::from(required(&r.params, "cwd")?);
        if !p.is_absolute() {
            return Err("Choose an absolute project path".into());
        }
        let p = p
            .canonicalize()
            .map_err(|e| format!("Cannot open project: {e}"))?;
        if !p.is_dir() {
            return Err("Project must be a directory".into());
        }
        if p == std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
            || p == std::path::Path::new("/")
        {
            return Err(
                "Choose a specific project directory, rather than your home or filesystem root"
                    .into(),
            );
        }
        p.to_string_lossy().into_owned()
    };
    if let Some(key) = &request_key
        && let Some(old) = s
            .tasks
            .iter()
            .find(|t| t.parent == r.parent && t.request_key.as_ref() == Some(key))
    {
        if old.prompt != prompt || old.provider != provider || old.model != model || old.cwd != cwd
        {
            return Err("Retry key was already used for a different request".into());
        }
        return Ok(old.public(true));
    }
    if cwd.len() > 1024 {
        return Err("Project path exceeds the 1,024 byte limit".into());
    }
    if s.tasks.iter().filter(|t| t.active()).count() >= 4 {
        return Err("Four tasks are already active; wait or cancel one".into());
    }
    if s.tasks.len() >= 1000 {
        return Err(
            "History is full. Export and clear tasks.json while the service is stopped".into(),
        );
    }
    let task = Task {
        id: id(),
        parent: r.parent,
        provider: provider.clone(),
        model,
        cwd,
        account_home: account_homes.get(&provider).cloned().flatten(),
        account_homes,
        prompt,
        status: "queued".into(),
        activity: "Resolving selected account".into(),
        result: String::new(),
        error: None,
        created_at: now(),
        completed_at: None,
        request_key,
        output_truncated: false,
    };
    let cancel = Arc::new(AtomicBool::new(false));
    s.cancel.insert(task.id.clone(), cancel.clone());
    s.tasks.push(task.clone());
    if let Err(e) = s.save() {
        s.tasks.pop();
        s.cancel.remove(&task.id);
        return Err(e);
    }
    drop(s);
    let shared = state.clone();
    let task_id = task.id.clone();
    thread::spawn(move || {
        let outcome = runner::run(&shared, &task_id, &cancel);
        let mut s = shared.lock().unwrap();
        let cancelled = s
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .is_some_and(|t| t.status == "cancelled");
        if !cancelled && let Some(t) = s.tasks.iter_mut().find(|t| t.id == task_id) {
            t.completed_at = Some(now());
            match outcome {
                Ok(()) => {
                    t.status = "completed".into();
                    t.activity = "Finished".into();
                }
                Err(e) => {
                    t.status = "failed".into();
                    t.activity = "Task failed".into();
                    t.error = Some(clipped(&e, 1500));
                }
            }
        }
        let reviewed = s
            .tasks
            .iter()
            .any(|t| t.parent.as_deref() == Some(&task_id) && t.status == "completed");
        if !cancelled
            && !reviewed
            && let Some(t) = s
                .tasks
                .iter_mut()
                .find(|t| t.id == task_id && t.parent.is_none() && t.status == "completed")
        {
            t.status = "failed".into();
            t.error = Some("Lead returned without a completed independent review; two-agent workflow is incomplete".into());
        }
        let unfinished = s
            .tasks
            .iter()
            .any(|t| t.parent.as_deref() == Some(&task_id) && t.active());
        if unfinished {
            cancel_tree(&mut s, &task_id);
            if let Some(t) = s.tasks.iter_mut().find(|t| t.id == task_id) {
                t.status = "failed".into();
                t.error=Some("Lead exited before collecting all child results; unfinished children were cancelled".into());
            }
        }
        s.capabilities.remove(&task_id);
        s.cancel.remove(&task_id);
        if let Err(e) = s.save() {
            eprintln!("Cannot persist task completion: {e}");
        }
    });
    Ok(task.public(true))
}

fn connection(state: Shared, mut socket: Box<dyn Stream>, token: String) {
    let result = (|| {
        let mut line = String::new();
        BufReader::new(&mut socket)
            .take(MAX_FRAME + 1)
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        if line.len() as u64 > MAX_FRAME {
            return Err("Request too large".into());
        }
        let envelope: Value =
            serde_json::from_str(&line).map_err(|e| format!("Invalid request: {e}"))?;
        if envelope["token"].as_str() != Some(&token) {
            return Err("Invalid local transport credential".into());
        }
        let r = serde_json::from_value(envelope["request"].clone())
            .map_err(|e| format!("Invalid request: {e}"))?;
        handle(&state, r)
    })();
    let response = match result {
        Ok(data) => json!({"ok":true,"data":data}),
        Err(error) => json!({"ok":false,"error":error}),
    };
    let _ = writeln!(socket, "{response}");
}
pub fn serve() -> Result<()> {
    private_dir(&directory())?;
    private_dir(&runtime())?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(runtime().join("broker.lock"))
        .map_err(|e| e.to_string())?;
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err("Agent Team is already running".into());
    }
    let path = directory().join("tasks.json");
    let mut tasks: Vec<Task> = if path.exists() {
        let data = fs::read(&path).map_err(|e| e.to_string())?;
        if data.len() > 64 * 1024 * 1024 {
            return Err("History exceeds 64 MiB".into());
        }
        serde_json::from_slice(&data)
            .map_err(|e| format!("Cannot load history; preserved without overwriting: {e}"))?
    } else {
        Vec::new()
    };
    for t in &mut tasks {
        if t.active() {
            t.status = "interrupted".into();
            t.completed_at = Some(now());
            t.activity = "Service restarted".into();
            t.error =
                Some("Previous execution was interrupted. Start a fresh task to retry.".into());
        }
    }
    let state = Arc::new(Mutex::new(State {
        tasks,
        cancel: HashMap::new(),
        capabilities: HashMap::new(),
    }));
    state.lock().unwrap().save()?;
    let path = runtime().join("broker.sock");
    let _ = fs::remove_file(&path);
    let unix = UnixListener::bind(&path);
    let tcp = if unix.is_err() {
        Some(TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).map_err(|e| e.to_string())?)
    } else {
        None
    };
    let token = id();
    let endpoint = if let Some(tcp) = &tcp {
        json!({"kind":"tcp","address":tcp.local_addr().map_err(|e|e.to_string())?.to_string(),"token":token})
    } else {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
        json!({"kind":"unix","token":token})
    };
    atomic_write(
        &runtime().join("endpoint.json"),
        &serde_json::to_vec(&endpoint).map_err(|e| e.to_string())?,
    )?;
    let connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    loop {
        let stream: Box<dyn Stream> = if let Some(listener) = &tcp {
            let (stream, _) = listener.accept().map_err(|e| e.to_string())?;
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .map_err(|e| e.to_string())?;
            stream
                .set_write_timeout(Some(Duration::from_secs(3)))
                .map_err(|e| e.to_string())?;
            Box::new(stream)
        } else {
            let (stream, _) = unix.as_ref().unwrap().accept().map_err(|e| e.to_string())?;
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .map_err(|e| e.to_string())?;
            stream
                .set_write_timeout(Some(Duration::from_secs(3)))
                .map_err(|e| e.to_string())?;
            Box::new(stream)
        };
        if connections.load(Ordering::SeqCst) >= 24 {
            continue;
        }
        connections.fetch_add(1, Ordering::SeqCst);
        let shared = state.clone();
        let count = connections.clone();
        let key = token.clone();
        thread::spawn(move || {
            connection(shared, stream, key);
            count.fetch_sub(1, Ordering::SeqCst);
        });
    }
}
