use std::sync::Arc;
use anyhow::{Result};
use async_trait::async_trait;
use diesel::prelude::*;
use futures::lock::Mutex;
use log::error;
use prew::{Reporter, PostgresqlPacket};
use prew::packet::Direction;
use prew::rule::AuthenticationContext;
use crate::models::reports::{NewReport, PostgresqlPacketType};

#[derive(Clone, Debug)]
struct ReporterContext {
    conn: Arc<Mutex<PgConnection>>,
}

#[derive(Clone, Debug)]
pub struct Context {
    authinfo: AuthenticationContext,
    reporter_context: ReporterContext,
}
impl prew::rule::WithAuthenticationContext for Context {
    fn authinfo(&mut self) -> &mut AuthenticationContext {
        &mut self.authinfo
    }
}
impl Context {
    pub fn new(conn_str: String) -> Result<Context> {
        let conn = Arc::new(Mutex::new(
            PgConnection::establish(&conn_str)?
        ));
        Ok(
            Context{
                authinfo: AuthenticationContext {
                    authenticated: false,
                    username: None,
                },
                reporter_context: ReporterContext {
                    conn,
                }
            }
        )
    }
}

#[derive(Clone)]
pub struct PostgresqlReporter {
}

impl PostgresqlReporter {
    pub fn new() -> PostgresqlReporter {
        PostgresqlReporter { }
    }
}
#[async_trait]
impl Reporter<PostgresqlPacket, Context> for PostgresqlReporter {
    fn report(
        &self,
        message: &PostgresqlPacket,
        direction: Direction,
        context: &Context
    ) -> Result<()> {
        let packet_info = serde_json::to_value(&message.info)?;
        let bytes = message.bytes.clone();
        let packet_type: PostgresqlPacketType = (&message.info).into();
        let authinfo = &context.authinfo;
        let username;
        if authinfo.authenticated {
            username = authinfo.username.clone();
        } else {
            username = None;
        }
        let conn = context.reporter_context.conn.clone();
        tokio::spawn(async move {
            let report = NewReport::create(
                username,
                packet_type,
                Some(direction.into()),
                Some(packet_info),
                bytes,
                false
            );
            {
                // mutex scope
                let mut h = conn.lock().await;
                if let Err(error) = report.commit(&mut h) {
                    error!("Unable to report on packet: {:?} - {:?}", &report.packet_info, &error);
                }
            }
        });
        Ok(())
    }

}
