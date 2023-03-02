use std::sync::Arc;
use anyhow::Result;
use prew::{PacketRules, RewriteReverseProxy, RuleSetProcessor};
use clap::Parser;
use futures::lock::Mutex;
use impulse::prew::{AppendUserNameTransformer, Context, ImpulseReporter};


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
    let transformer = AppendUserNameTransformer::new();
    let encoder = prew::MessageEncoder::new();
    let reporter = ImpulseReporter::new();
    let report_connstr = args.report_connstr;
    let create_context = move || {
        Context::new(report_connstr.clone()).unwrap()
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
        bind_addr: args.bind_addr,
        server_addr: args.server_addr,
        processor,
    };
    proxy.add_proxy(Box::new(packet_rules)).await;
    proxy.run().await;
    Ok(())
}