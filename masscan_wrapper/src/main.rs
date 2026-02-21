use clap::Parser;
use reqwest::{Client, cookie::Jar};
use serde_json::json;
use std::{process::Command, sync::Arc};

#[derive(Parser, Debug)]
#[command(author, version, about = "Run masscan on port 25565 and post IPs to API")]
struct Args {
    /// API base endpoint (e.g. https://example.com)
    #[arg(short, long)]
    endpoint: String,

    /// Password for API login
    #[arg(short, long)]
    password: String,

    /// Target range to scan (e.g. 0.0.0.0/0)
    #[arg(short, long, default_value = "0.0.0.0/0")]
    range: String,

    /// Masscan rate (packets per second)
    #[arg(short = 'R', long, default_value = "10000")]
    rate: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Build client with cookie store
    let jar = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(jar.clone())
        .build()?;

    // Login
    println!("[*] Logging in to {}...", args.endpoint);
    let login_url = format!("{}/api/v1/auth/login", args.endpoint);
    let res = client
        .post(&login_url)
        .json(&json!({ "password": args.password }))
        .send()
        .await?;

    if !res.status().is_success() {
        eprintln!("[-] Login failed: {}", res.status());
        std::process::exit(1);
    }
    println!("[+] Logged in successfully.");

    // Run masscan
    println!("[*] Running masscan on {} port 25565...", args.range);
    let output = Command::new("masscan")
        .args([
            &args.range,
            "-p25565",
            "--rate",
            &args.rate.to_string(),
            "-oG", // grepable output
            "-",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[-] masscan failed: {}", stderr);
        std::process::exit(1);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ips = parse_masscan_output(&stdout);
    println!("[+] Found {} IPs.", ips.len());

    // Post each IP
    let add_url = format!("{}/api/v1/ip/add", args.endpoint);
    let mut success = 0;
    let mut failed = 0;

    for ip in &ips {
        let res = client
            .post(&add_url)
            .json(&json!({ "ip": ip }))
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => {
                println!("[+] Added {}", ip);
                success += 1;
            }
            Ok(r) => {
                eprintln!("[-] Failed to add {} - status: {}", ip, r.status());
                failed += 1;
            }
            Err(e) => {
                eprintln!("[-] Error adding {}: {}", ip, e);
                failed += 1;
            }
        }
    }

    println!("\n[*] Done. {} added, {} failed.", success, failed);
    Ok(())
}

/// Parse masscan grepable output lines like:
/// Host: 1.2.3.4 ()	Ports: 25565/open/tcp//minecraft///
fn parse_masscan_output(output: &str) -> Vec<String> {
    let mut ips = Vec::new();
    for line in output.lines() {
        if line.starts_with("Host:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                ips.push(parts[1].to_string());
            }
        }
    }
    ips
}