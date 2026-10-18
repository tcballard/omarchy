use crate::{model::*, rpc};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};

fn tools() -> Value {
    json!([
        {"name":"orchestrator_capabilities","description":"Discover the installed Codex/Claude execution adapters. All tasks are read-only and use this machine's selected subscription account.","inputSchema":{"type":"object","properties":{},"additionalProperties":false}},
        {"name":"delegate_task","description":"Start an independent read-only reviewer on Codex or Claude. Give a self-contained brief and stable clientRequestId; no conversation history is inherited. Call task_status until terminal before completing your response.","inputSchema":{"type":"object","properties":{"provider":{"type":"string","enum":["codex","claude"]},"task":{"type":"string","maxLength":12000},"model":{"type":"string"},"clientRequestId":{"type":"string","maxLength":128}},"required":["provider","task","clientRequestId"],"additionalProperties":false}},
        {"name":"task_status","description":"Read your delegated task's status and result. Optionally wait up to 10 seconds. A timeout leaves the task running. Only your own children can be read.","inputSchema":{"type":"object","properties":{"taskId":{"type":"string"},"waitMs":{"type":"integer","minimum":0,"maximum":10000}},"required":["taskId"],"additionalProperties":false}},
        {"name":"task_cancel","description":"Cancel one of your own delegated tasks.","inputSchema":{"type":"object","properties":{"taskId":{"type":"string"}},"required":["taskId"],"additionalProperties":false}}
    ])
}
fn call(name: &str, args: Value, parent: &str, capability: &str) -> Result<Value> {
    let (op, allowed) = match name {
        "orchestrator_capabilities" => ("doctor", vec![]),
        "delegate_task" => (
            "start",
            vec!["provider", "task", "model", "clientRequestId"],
        ),
        "task_status" => ("read", vec!["taskId", "waitMs"]),
        "task_cancel" => ("cancel", vec!["taskId"]),
        _ => return Err("Unknown tool".into()),
    };
    let object = args.as_object().ok_or("Tool arguments must be an object")?;
    if object.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("Unsupported tool argument".into());
    }
    if name == "delegate_task" {
        required(&args, "clientRequestId")?;
    }
    rpc(Request {
        op: op.into(),
        params: args,
        parent: Some(parent.into()),
        capability: Some(capability.into()),
    })
}
pub fn serve() -> Result<()> {
    let parent = std::env::var("AGENT_TEAM_TASK")
        .map_err(|_| "MCP bridge requires a broker-owned task scope")?;
    let capability =
        std::env::var("AGENT_TEAM_CAPABILITY").map_err(|_| "Missing task capability")?;
    let stdin = std::io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let mut out = std::io::stdout().lock();
    loop {
        let mut line = String::new();
        let n = input
            .by_ref()
            .take(MAX_FRAME + 1)
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(());
        }
        if n as u64 > MAX_FRAME {
            return Err("MCP frame exceeds 256 KiB".into());
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                writeln!(out,"{}",json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}})).map_err(|e|e.to_string())?;
                out.flush().map_err(|e| e.to_string())?;
                continue;
            }
        };
        let Some(id) = request.get("id") else {
            continue;
        };
        let method = request["method"].as_str().unwrap_or("");
        let result = match method {
            "initialize" => Ok(
                json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"omarchy-agent-team","version":"0.0.1"},"instructions":"Read-only delegation across local signed-in CLIs. Always collect child results before finishing."}),
            ),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({"tools":tools()})),
            "tools/call" => {
                let result = call(
                    request["params"]["name"].as_str().unwrap_or(""),
                    request["params"]["arguments"]
                        .as_object()
                        .map(|a| Value::Object(a.clone()))
                        .unwrap_or(json!({})),
                    &parent,
                    &capability,
                );
                Ok(match result {
                    Ok(data) => {
                        json!({"content":[{"type":"text","text":data.to_string()}],"isError":false})
                    }
                    Err(e) => json!({"content":[{"type":"text","text":e}],"isError":true}),
                })
            }
            _ => Err("Method not found".to_string()),
        };
        let response = match result {
            Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
            Err(message) => {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":message}})
            }
        };
        writeln!(out, "{response}")
            .and_then(|_| out.flush())
            .map_err(|e| e.to_string())?;
    }
}
