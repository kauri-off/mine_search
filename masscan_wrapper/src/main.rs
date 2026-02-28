use clap::Parser;
use reqwest::{Client, cookie::Jar};
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Import IPs from a masscan output file and post them to the API"
)]
struct Args {
    /// API base endpoint (e.g. https://example.com)
    #[arg(short, long)]
    endpoint: String,

    /// Password for API login
    #[arg(short, long)]
    password: String,

    /// Path to masscan -oG output file
    #[arg(short, long)]
    file: String,
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

    println!("[*] Reading masscan output from {}...", args.file);
    let contents = std::fs::read_to_string(&args.file)?;
    let ips = parse_masscan_output(&contents);

    println!("[+] Found {} IPs.", ips.len());

    if ips.is_empty() {
        println!("[*] Nothing to do.");
        println!(
            "[!] Tip: Generate an input file with:\n    \
             sudo masscan <range> -p25565 --rate 10000 -oG output.txt\n    \
             Then re-run this tool with --file output.txt"
        );
        return Ok(());
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

    let add_url = format!("{}/api/v1/target/add_list", args.endpoint);
    let body: Vec<Value> = ips
        .into_iter()
        .map(|ip| json!({ "ip": ip, "quick": false }))
        .collect();
    client.post(&add_url).json(&body).send().await?;

    println!("[*] Done.");
    Ok(())
}

/// Parse masscan grepable output lines like:
/// Timestamp: 1771711220	Host: 192.168.1.3 ()	Ports: 25565/open/tcp//unknown//
fn parse_masscan_output(output: &str) -> Vec<String> {
    let mut ips = Vec::new();
    for line in output.lines() {
        if line.starts_with('#') {
            continue;
        }
        if let Some(host_pos) = line.find("Host: ") {
            let after_host = &line[host_pos + 6..];
            if let Some(ip) = after_host.split_whitespace().next() {
                ips.push(ip.to_string());
            }
        }
    }
    ips
}
