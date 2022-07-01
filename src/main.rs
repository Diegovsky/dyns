use std::time::Duration;

use anyhow::Context;
use isahc::{Request, Body, ReadResponseExt};


#[derive(Clone, Debug, serde::Deserialize)]
struct Record {
    name: String,
    proxy: bool
}

#[derive(Clone, Debug, serde::Deserialize)]
struct Config {
    zone_id: String,
    email: String,
    auth_key: String,
    authorization: String,
    records: Vec<Record>,
}

static CONFIG_FILE: &str = "/etc/dyns.toml";

#[derive(Clone, Debug, serde::Deserialize)]
struct RecordInfo {
    id: String,
    name: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct CloudflareResponse {
    success: bool,
    errors: Vec<String>,
    messages: Vec<String>,
    result: Vec<RecordInfo>
}

/// Taken from https://api.cloudflare.com/#dns-records-for-a-zone-patch-dns-record
#[derive(Clone, Debug, serde::Serialize)]
struct UpdateRecordBody<'a> {
    /* #[serde(rename="type")]
    type_: String, */
    content: &'a str,
    proxy: bool,
}

fn get_dns_record_id(client: &mut isahc::HttpClient, cfg: &Config, name: &str) -> anyhow::Result<String> {
    let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", cfg.zone_id);
    let mut response = client.send(Request::get(url)
                .header("X-auth-email", &cfg.email)
                .header("x-auth-key", &cfg.auth_key)
                .body(Body::empty()).expect("Failed to create request"))?;
    let body: CloudflareResponse = response.json().expect("Failed to parse response");
    if !body.success {
        anyhow::bail!("Failed to get DNS record ID: {:?}", body.errors)
    }
    body.result.into_iter()
        .find(|info| info.name == name)
        .map(|info| info.id)
        .ok_or(anyhow::anyhow!("Did not find any DNS record with name {}", name))

}



fn update_record(client: &mut isahc::HttpClient, cfg: &Config, record: &Record, ip: &str) -> anyhow::Result<()> {
    let record_id = get_dns_record_id(client, cfg, &record.name)?;
    let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", cfg.zone_id, record_id);
    client.send(Request::patch(url)
                .header("X-auth-email", &cfg.email)
                .header("x-auth-key", &cfg.auth_key)
                .header("Authorization", format!("Bearer {}", cfg.authorization))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&UpdateRecordBody {
                    content: ip,
                    proxy: record.proxy,
                }).expect("Failed to serialize request body"))).expect("Failed to create a request"))?;

    log::info!("Successfully updated record {} to {}", record.name, ip);
    Ok(())
}

fn get_current_ip(client: &mut isahc::HttpClient) -> anyhow::Result<String> {
    Ok(client.get("https://api.ipify.org/").context("Failed to get new IP address")?
       .text()
       .map(|t| t.trim().to_string())?)

        
}
use clap::Parser;
use simplelog::{CombinedLogger, SimpleLogger, WriteLogger};
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    config: Option<String>,
}

fn init_logger() {
    use log::LevelFilter;
    use simplelog::Config;
    let queue = userv_foundation::telsend::open_queue().expect("Failed to open userv queue");
    CombinedLogger::init(vec![
                         SimpleLogger::new(LevelFilter::Info, Config::default()),
                         WriteLogger::new(LevelFilter::Error, Config::default(), queue)]).unwrap();
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_logger();

    let cfg = std::fs::read_to_string(cli.config.as_deref().unwrap_or(CONFIG_FILE))?;
    let cfg = toml::from_str::<Config>(&cfg)?;
    let records = &cfg.records;
    let mut client = isahc::HttpClient::new()?;
    let mut ip = get_current_ip(&mut client)?;
    loop {
        for record in records {
            if let Err(e) = update_record(&mut client, &cfg, record, &ip) {
                log::error!("An error happened while updating record for {}: {}", record.name, e);
                return Err(e);
            }
        }
        loop {
            std::thread::sleep(Duration::from_secs(5*60));
            let new_ip = get_current_ip(&mut client)?;
            if new_ip != ip {
                ip = new_ip;
                break;
            }
            log::info!("IP hasn't changed, sleeping...");
        }
    }
}
