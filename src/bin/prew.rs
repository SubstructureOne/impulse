use std::sync::Arc;
use anyhow::Result;
use prew::{PacketRules, PrewRuleSet, RewriteReverseProxy};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct PrewArgs {
    #[arg(short, long)]
    bind_addr: String,
    #[arg(short, long)]
    server_addr: String,
    #[arg(short, long)]
    report_connstr: String,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = PrewArgs::parse();
    let parser = prew::PostgresParser::new();
    let filter = prew::NoFilter::new();
    let transformer = prew::NoTransform::new();
    let encoder = prew::MessageEncoder::new();
    let reporter = prew::NoReport::new();
    let prew_rules = PrewRuleSet::new(
        &parser,
        &filter,
        &transformer,
        &encoder,
        &reporter
    );
    let processor = Arc::new(prew_rules);
    let mut proxy = RewriteReverseProxy::new();
    let packet_rules = PacketRules {
        bind_addr: args.bind_addr,
        server_addr: args.server_addr,
        processor,
    };
    proxy.add_proxy(Box::new(packet_rules)).await;
    proxy.run(args.report_connstr).await;
    Ok(())
}