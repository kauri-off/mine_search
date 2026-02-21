use clap::Parser;
use reqwest::{Client, cookie::Jar};
use serde_json::{Value, json};
use std::{process::Command, sync::Arc};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Run masscan on port 25565 and post IPs to API"
)]
struct Args {
    /// API base endpoint (e.g. https://example.com)
    #[arg(short, long)]
    endpoint: String,

    /// Password for API login
    #[arg(short, long)]
    password: String,

    #[command(subcommand)]
    command: SubCommand,
}

#[derive(clap::Subcommand, Debug)]
enum SubCommand {
    /// Run masscan automatically and send results to the API
    Scan {
        /// Target range to scan (e.g. 192.168.1.0/24)
        #[arg(short, long)]
        range: String,

        /// Masscan rate (packets per second)
        #[arg(short = 'R', long, default_value = "10000")]
        rate: u32,
    },
    /// Import IPs from a previously saved masscan grepable output file
    Import {
        /// Path to masscan -oG output file
        #[arg(short, long)]
        file: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Build client with cookie store
    let jar = Arc::new(Jar::default());
    let client = Client::builder().cookie_provider(jar.clone()).build()?;

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

    // Get IPs either from file or by running masscan
    let ips = match &args.command {
        SubCommand::Import { file } => {
            println!("[*] Reading masscan output from {}...", file);
            let contents = std::fs::read_to_string(file)?;
            parse_masscan_output(&contents)
        }
        SubCommand::Scan { range, rate } => {
            println!("[*] Running masscan on {} port 25565...", range);
            let output = Command::new("masscan")
                .args([
                    range.as_str(),
                    "-p25565",
                    "--rate",
                    &rate.to_string(),
                    "-oG",
                    "-",
                ])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("[-] masscan failed: {}", stderr);
                std::process::exit(1);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_masscan_output(&stdout)
        }
    };
    println!("[+] Found {} IPs.", ips.len());

    if ips.is_empty() {
        println!("[*] Nothing to do.");
        return Ok(());
    }

    println!("\nIPs to be added:");
    for ip in &ips {
        println!("  {}", ip);
    }

    print!(
        "\nAdd these {} IP(s) to {}? [y/N] ",
        ips.len(),
        args.endpoint
    );
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut input)?;
    if input.trim().to_lowercase() != "y" {
        println!("[-] Aborted.");
        return Ok(());
    }

    let add_url = format!("{}/api/v1/ip/add_list", args.endpoint);

    let ips: Vec<Value> = ips.into_iter().map(|ip| json!({ "ip": ip })).collect();
    client.post(&add_url).json(&ips).send().await?;

    println!("[*] Done.");
    Ok(())
}

/// Parse masscan grepable output lines like:
/// Timestamp: 1771711220	Host: 192.168.1.3 ()	Ports: 25565/open/tcp//unknown//
fn parse_masscan_output(output: &str) -> Vec<String> {
    let mut ips = Vec::new();
    for line in output.lines() {
        // Skip comment lines
        if line.starts_with('#') {
            continue;
        }
        // Find "Host: <ip>" anywhere in the line
        if let Some(host_pos) = line.find("Host: ") {
            let after_host = &line[host_pos + 6..];
            // IP is the next whitespace-delimited token
            if let Some(ip) = after_host.split_whitespace().next() {
                ips.push(ip.to_string());
            }
        }
    }
    ips
}
