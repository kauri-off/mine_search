use clap::Parser;
use proto::api::{AddAddrRequest, AddTargetListRequest, LoginRequest, api_client::ApiClient};
use tonic::{
    Request,
    transport::{Channel, ClientTlsConfig},
};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Import IPs from a masscan output file and submit them to the backend over gRPC"
)]
struct Args {
    /// Backend gRPC endpoint, e.g. http://127.0.0.1:3000 or https://example.com:443
    #[arg(short, long)]
    endpoint: String,

    /// Password for API login
    #[arg(short, long)]
    password: String,

    /// Path to masscan -oG output file
    #[arg(short, long)]
    file: String,
}

async fn connect(endpoint: &str) -> anyhow::Result<Channel> {
    let mut ep = Channel::from_shared(endpoint.to_string())?;
    if endpoint.starts_with("https") {
        ep = ep.tls_config(ClientTlsConfig::new().with_webpki_roots())?;
    }
    Ok(ep.connect().await?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("[*] Connecting to {}...", args.endpoint);
    let channel = connect(&args.endpoint).await?;
    let mut client = ApiClient::new(channel);

    println!("[*] Logging in...");
    let token = client
        .login(LoginRequest {
            password: args.password,
        })
        .await
        .map_err(|e| anyhow::anyhow!("login failed: {}", e.message()))?
        .into_inner()
        .token;
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

    print!("\nAdd these {} IP(s) to {}? [y/N] ", ips.len(), args.endpoint);
    std::io::Write::flush(&mut std::io::stdout())?;
    let mut input = String::new();
    std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut input)?;
    if input.trim().to_lowercase() != "y" {
        println!("[-] Aborted.");
        return Ok(());
    }

    let targets = ips
        .into_iter()
        .map(|ip| AddAddrRequest {
            addr: ip,
            quick: false,
        })
        .collect();

    // Attach the session token as Bearer metadata.
    let mut req = Request::new(AddTargetListRequest { targets });
    req.metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse()?);
    client.add_target_list(req).await?;

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
