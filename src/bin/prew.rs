use std::sync::Arc;
use std::env;
use std::fs;

use anyhow::{Context, Result};
use clap::Parser;
use futures::lock::Mutex;
use log::info;
use prew::{PacketRules, RewriteReverseProxy, RuleSetProcessor};
use serde::{Serialize, Deserialize};

use impulse::prew::{AppendUserNameTransformer, ImpulseReporter};


#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct PrewArgs {
    #[arg(short, long)]
    bind_addr: Option<String>,
    #[arg(short, long)]
    server_addr: Option<String>,
    #[arg(short, long)]
    report_connstr: Option<String>,
    #[arg(short, long)]
    config_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrewConfig {
    bind_addr: Option<String>,
    server_addr: Option<String>,
    report_connstr: Option<String>,
}

fn parse_config(config_file: Option<String>) -> Result<Option<PrewConfig>> {
    match config_file {
        Some(path) => {
            let cfg_str = fs::read_to_string(&path)?;
            Ok(toml::from_str(&cfg_str)?)
        }
        None => Ok(None)
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    env_logger::init();
    dotenvy::dotenv().ok();
    let mut args = PrewArgs::parse();
    let opt_config = parse_config(args.config_file)?;
    if let Some(config) = opt_config {
        println!("Loaded config: {:?}", &config);
        args.bind_addr = args.bind_addr.or(config.bind_addr);
        args.server_addr = args.server_addr.or(config.server_addr);
        args.report_connstr = args.report_connstr.or(config.report_connstr);
    }
    let parser = prew::PostgresParser::new();
    let filter = prew::NoFilter::new();
    let transformer = AppendUserNameTransformer::new();
    let encoder = prew::MessageEncoder::new();
    let reporter = ImpulseReporter::new();
    let report_connstr = args.report_connstr.or(env::var("DATABASE_URL").ok())
        .context("No impulse database connection string specified")?;
    let server_addr = args.server_addr.context("No server address specified")?;
    let create_context = move || {
        impulse::prew::Context::new(report_connstr.clone()).unwrap()
    };
    let prew_rules = RuleSetProcessor::new(
        &parser,
        &filter,
        &transformer,
        &encoder,
        &reporter,
        &create_context,
    );
    let processor = Arc::new(Mutex::new(prew_rules));
    let mut proxy = RewriteReverseProxy::new();
    let packet_rules = PacketRules {
        bind_addr: args.bind_addr.context("Bind address not specified")?,
        server_addr,
        processor,
    };
    proxy.add_proxy(Box::new(packet_rules)).await;
    info!("Starting proxy");
    proxy.run().await;
    Ok(())
}