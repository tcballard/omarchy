use crate::{model::*, runner};
use serde_json::Value;
use std::{path::Path, process::Command};

pub fn args(task: &Task) -> Vec<String> {
    let mut args: Vec<String> = [
        "--prompt-file",
        "/dev/stdin",
        "--output-format",
        "streaming-messages-json",
        "--sandbox",
        "read-only",
        "--permission-mode",
        "dontAsk",
        "--tools",
        "read_file,grep,list_dir",
        "--disallowed-tools",
        "Agent",
        "--deny",
        "MCPTool",
        "--allow",
        "Read",
        "--allow",
        "Grep",
        "--no-subagents",
        "--no-plan",
        "--no-memory",
        "--no-leader",
        "--no-auto-update",
        "--disable-web-search",
        "--max-turns",
        "60",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    if let Some(model) = &task.model {
        args.extend(["--model".into(), model.clone()]);
    }
    args
}

pub fn environment(command: &mut Command) {
    for name in [
        "GROK_CONFIG",
        "GROK_CONFIG_PATH",
        "XAI_API_KEY",
        "GROK_API_KEY",
        "GROK_API_BASE_URL",
        "GROK_AGENT_SECRET",
        "GROK_SANDBOX",
        "GROK_AUTH_TOKEN",
        "GROK_WS_URL",
        "GROK_WS_ORIGIN",
    ] {
        command.env_remove(name);
    }
    for name in [
        "GROK_CLAUDE_MCPS_ENABLED",
        "GROK_CURSOR_MCPS_ENABLED",
        "GROK_CLAUDE_HOOKS_ENABLED",
        "GROK_CURSOR_HOOKS_ENABLED",
        "GROK_MANAGED_MCPS_ENABLED",
        "GROK_MANAGED_MCP_GATEWAY_TOOLS_ENABLED",
    ] {
        command.env(name, "false");
    }
}

pub fn validate_report(report: &Value, cwd: &str) -> Result<()> {
    if report["cwd"].as_str() != Some(cwd) || !report["grokVersion"].is_string() {
        return Err("Grok inspect returned an incompatible project/version report".into());
    }
    let layers = report["configSources"]["layers"]
        .as_array()
        .ok_or("Grok inspect lacks config layer inventory")?;
    if layers
        .iter()
        .any(|l| l.get("note").is_some_and(|n| n.as_str() != Some("empty")))
    {
        return Err("Grok configuration has unreadable or invalid layers; review refused".into());
    }
    for field in ["hooks", "mcpServers", "plugins", "lspServers"] {
        let entries = report[field]
            .as_array()
            .ok_or("Grok inspect lacks extension inventory; update the native CLI")?;
        let active = entries.iter().any(|entry| match field {
            "plugins" => entry["enabled"] != false,
            "lspServers" => entry["untrusted"] != true,
            _ => entry["disabled"] != true,
        });
        if active {
            return Err(format!(
                "Grok review refused active {field} in this project/account. This trial requires an extension-free Grok configuration; your configuration was not changed."
            ));
        }
    }
    for field in ["mcpConfigProblems", "configWarnings"] {
        if let Some(value) = report.get(field) {
            let list = value
                .as_array()
                .ok_or("Grok inspect returned malformed diagnostics")?;
            if !list.is_empty() {
                return Err(
                    "Grok configuration has warnings; resolve them before running a review".into(),
                );
            }
        }
    }
    Ok(())
}

pub fn preflight(path: &Path, task: &Task) -> Result<()> {
    let mut help = Command::new(path);
    help.arg("--help").current_dir(&task.cwd);
    runner::account_env(&mut help, task);
    let help = runner::probe(help)?;
    for flag in [
        "--prompt-file",
        "--output-format",
        "--sandbox",
        "--permission-mode",
        "--tools",
        "--disallowed-tools",
        "--deny",
        "--max-turns",
    ] {
        if !help.contains(flag) {
            return Err(format!(
                "Grok CLI lacks {flag}; this build cannot run it safely"
            ));
        }
    }
    let mut inspect = Command::new(path);
    inspect.args(["inspect", "--json"]).current_dir(&task.cwd);
    runner::account_env(&mut inspect, task);
    let report: Value = serde_json::from_str(&runner::probe(inspect)?)
        .map_err(|_| "Grok inspect did not return the required JSON inventory")?;
    validate_report(&report, &task.cwd)
}

pub fn terminal(value: &Value) -> Result<bool> {
    if value["type"] != "result" {
        return Ok(false);
    }
    if value["is_error"] != false
        || value["subtype"] != "success"
        || value["stop_reason"] != "end_turn"
    {
        return Err(
            "Grok did not finish successfully (error, refusal, cancellation, or turn limit)".into(),
        );
    }
    if value["result"].as_str().is_none_or(|s| s.trim().is_empty()) {
        return Err("Grok returned no final review text".into());
    }
    Ok(true)
}
