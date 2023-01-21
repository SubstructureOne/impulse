use std::rc::Rc;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use log::{debug, info, trace};

use super::ManagementConfig;
use super::postgres::PostgresManager;
use crate::models::charges::{Charge, NewTimeCharge, TimeChargeType};
use crate::models::reports::{ReportToCharge};
use crate::models::transactions::NewTransaction;
use crate::models::users::User;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct ImpulseArgs {
    #[arg(short, long)]
    generate_charges: bool,
    #[arg(short, long)]
    generate_transactions: bool,
    #[arg(short, long)]
    process_timecharges: bool,
    #[arg(short, long)]
    compute_storage: bool,
}

pub async fn impulse(args: &ImpulseArgs) -> Result<()> {
    let mut impulse_conn = crate::connect_impulse_db()?;

    if args.process_timecharges {
        info!("Processing time charges");
        let mut count = 0;
        for user in User::all(&mut impulse_conn)? {
            let charges = Charge::from_timecharges_for_user(
                &mut impulse_conn,
                &user.user_id,
                None,
            )?;
            debug!("Generated {} charges for user {}", charges.len(), &user.user_id);
            count += charges.len();
        }
        info!("Created {} new charges from timecharges", count);
    }
    if args.generate_charges {
        info!("Generating charges from reports");
        let uncharged = ReportToCharge::uncharged(&mut impulse_conn)?;
        let charges = Charge::from_reports(&mut impulse_conn, uncharged)?;
        info!("Generated {} charges", charges.len());
    }
    if args.generate_transactions {
        info!("Generating transactions");
        let charges = Charge::untransacted(&mut impulse_conn)?;
        let transactions = NewTransaction::from_charges(
            &mut impulse_conn,
            &charges
        )?;
        info!("Generated {} transactions", transactions.len());
    }
    if args.compute_storage {
        // Intentionally compute storage last. Since timecharges are scaled
        // by time to create charges, if we create timecharges first, then we
        // will end up with additional tiny charges for every new timecharge
        // created multiplied by the time delta between the timecharge creation
        // and the charge creation.
        info!("Computing user storage");
        let config = Rc::new(ManagementConfig::from_env()?);
        let manager = PostgresManager::new(config.clone());
        let user2bytes = manager.compute_storage()?;
        // use a single timestamp for all timecharges for simpler querying
        let timecharge_time = Utc::now();
        for (user_id, quantity_bytes) in user2bytes {
            debug!("{}: {} bytes", &user_id, quantity_bytes);
            let timecharge = NewTimeCharge::create(
                user_id,
                Some(timecharge_time),
                TimeChargeType::DataStorageBytes,
                quantity_bytes as f64,
            ).commit(&mut impulse_conn)?;
            trace!("Created timecharge: {:?}", &timecharge);
        }
    }
    Ok(())
}
