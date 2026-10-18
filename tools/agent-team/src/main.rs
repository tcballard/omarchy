mod broker;
mod grok;
mod mcp;
mod model;
mod runner;

use model::{MAX_FRAME, Request, Result};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    os::unix::net::UnixStream,
    time::Duration,
};

pub fn rpc(request: Request) -> Result<Value> {
    let metadata = std::fs::read(model::runtime().join("endpoint.json")).map_err(|_| {
        "Agent Team is stopped. Open the panel and choose Start service.".to_string()
    })?;
    let endpoint: Value = serde_json::from_slice(&metadata).map_err(|e| e.to_string())?;
    let mut stream: Box<dyn model::Stream> = if endpoint["kind"] == "tcp" {
        let address: std::net::SocketAddr = endpoint["address"]
            .as_str()
            .ok_or("Missing broker address")?
            .parse()
            .map_err(|_| "Invalid broker address")?;
        if !address.ip().is_loopback() {
            return Err("Broker must be local".into());
        }
        let stream = TcpStream::connect_timeout(&address, Duration::from_secs(3))
            .map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| e.to_string())?;
        Box::new(stream)
    } else {
        let stream = UnixStream::connect(model::runtime().join("broker.sock"))
            .map_err(|_| "Agent Team is stopped. Start the service.".to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| e.to_string())?;
        Box::new(stream)
    };
    serde_json::to_writer(
        &mut stream,
        &json!({"token":endpoint["token"],"request":request}),
    )
    .map_err(|e| e.to_string())?;
    stream.write_all(b"\n").map_err(|e| e.to_string())?;
    let mut line = String::new();
    BufReader::new(stream)
        .take(MAX_FRAME + 1)
        .read_line(&mut line)
        .map_err(|e| e.to_string())?;
    if line.len() as u64 > MAX_FRAME {
        return Err("Response exceeded the protocol limit".into());
    }
    let response: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
    if response["ok"] == true {
        Ok(response["data"].clone())
    } else {
        Err(response["error"].as_str().unwrap_or("Broker error").into())
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");
    match command {
        "daemon" => broker::serve(),
        "mcp" => mcp::serve(),
        "request" => {
            let mut input = String::new();
            std::io::stdin()
                .take(MAX_FRAME + 1)
                .read_to_string(&mut input)
                .map_err(|e| e.to_string())?;
            if input.len() as u64 > MAX_FRAME {
                return Err("Request too large".into());
            }
            let r: Request = serde_json::from_str(&input).map_err(|e| e.to_string())?;
            println!("{}", json!({"ok":true,"data":rpc(r)?}));
            Ok(())
        }
        "list" | "doctor" => {
            let data = rpc(Request {
                op: command.into(),
                params: json!({}),
                parent: None,
                capability: None,
            })?;
            println!("{}", json!({"ok":true,"data":data}));
            Ok(())
        }
        "start" => {
            if args.len() < 4 {
                return Err("Usage: omarchy-agent-team start <codex|claude|default> /absolute/project \"review brief\" [codex|claude|grok]".into());
            }
            let data = rpc(Request {
                op: "start".into(),
                params: json!({"provider":args[1],"cwd":args[2],"task":args[3],"reviewer":args.get(4)}),
                parent: None,
                capability: None,
            })?;
            println!("{}", json!({"ok":true,"data":data}));
            Ok(())
        }
        "read" | "cancel" => {
            let id = args.get(1).ok_or("Task ID required")?;
            let data = rpc(Request {
                op: command.into(),
                params: json!({"taskId":id}),
                parent: None,
                capability: None,
            })?;
            println!("{}", json!({"ok":true,"data":data}));
            Ok(())
        }
        _ => {
            println!(
                "Agent Team 0.0.1\nCommands: daemon, doctor, list, start <provider> <project> <brief>, read <id>, cancel <id>, mcp\nRead-only delegation prototype. Start the user service with ./scripts/setup."
            );
            Ok(())
        }
    }
}
fn main() {
    if let Err(e) = run() {
        println!("{}", json!({"ok":false,"error":model::clipped(&e,1500)}));
        std::process::exit(1)
    }
}
