use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use futures::lock::Mutex;
use log::{debug, error};
use pg_query::NodeMut;

use prew::{Reporter, PostgresqlPacket, Transformer};
use prew::packet::Direction;
use prew::rule::AuthenticationContext;
use prew::postgresql::{PostgresqlPacketInfo, QueryMessage};

use crate::models::reports::{NewReport, PostgresqlPacketType};

#[derive(Clone, Debug)]
struct ReporterContext {
    pool: Pool<ConnectionManager<PgConnection>>,
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
        let manager = ConnectionManager::<PgConnection>::new(conn_str);
        let pool = Pool::builder().build(manager)?;
        // let conn = Arc::new(Mutex::new(
        //     PgConnection::establish(&conn_str)?
        // ));
        Ok(
            Context{
                authinfo: AuthenticationContext {
                    authenticated: false,
                    username: None,
                },
                reporter_context: ReporterContext {
                    pool,
                }
            }
        )
    }
}

#[derive(Clone)]
pub struct ImpulseReporter {
}

impl ImpulseReporter {
    pub fn new() -> ImpulseReporter {
        ImpulseReporter { }
    }
}
#[async_trait]
impl Reporter<PostgresqlPacket, Context> for ImpulseReporter {
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
        let mut conn = context.reporter_context.pool.get()?;
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
                // let mut h = conn.lock().await;
                if let Err(error) = report.commit(&mut conn) {
                    error!("Unable to report on packet: {:?} - {:?}", &report.packet_info, &error);
                }
            }
        });
        Ok(())
    }
}

#[derive(Clone)]
pub struct AppendUserNameTransformer {
}

impl AppendUserNameTransformer {
    pub fn new() -> AppendUserNameTransformer {
        AppendUserNameTransformer {}
    }

    unsafe fn transform_stmt(&self, node: NodeMut, username: &str) -> bool {
        let mut modified = false;
        match node {
            NodeMut::CreatedbStmt(stmt) => {
                let new_dbname = self.transform_dbname(&(*stmt).dbname, username);
                (*stmt).dbname = new_dbname;
                modified = true;
            }
            NodeMut::DropdbStmt(stmt) => {
                let new_dbname = self.transform_dbname(&(*stmt).dbname, username);
                (*stmt).dbname = new_dbname;
                modified = true;
            }
            NodeMut::AlterDatabaseSetStmt(stmt) => {
                let new_dbname = self.transform_dbname(&(*stmt).dbname, username);
                (*stmt).dbname = new_dbname;
                modified = true;
            },
            NodeMut::AlterDatabaseStmt(stmt) => {
                let new_dbname = self.transform_dbname(&(*stmt).dbname, username);
                (*stmt).dbname = new_dbname;
                modified = true;
            }
            _ => {}
        }
        modified
    }

    fn transform_dbname(&self, database_name: &str, username: &str) -> String {
        if database_name.eq(username) {
            // don't modify database name if the user is connecting to
            // or modifying their own database
            database_name.to_string()
        } else {
            format!("{}__{}", database_name, username)
        }
    }
}

impl Transformer<PostgresqlPacket, Context> for AppendUserNameTransformer {
    fn transform(&self, packet: &PostgresqlPacket, context: &Context) -> Result<PostgresqlPacket> {
        if let PostgresqlPacketInfo::Startup(message) = &packet.info {
            let dbname = message.get_parameter("database")
                .ok_or_else(|| anyhow!("Database name missing from startup message"))?;
            let username = message.get_parameter("user")
                .ok_or_else(|| anyhow!("Username missing from startup message"))?;
            let newdbname = self.transform_dbname(&dbname, &username);
            let mut message = message.clone();
            message.set_parameter("database", newdbname);
            Ok(PostgresqlPacket { info: PostgresqlPacketInfo::Startup(message), bytes: None })
        } else if let PostgresqlPacketInfo::Query(message) = &packet.info {
            if let Some(username) = &context.authinfo.username {
                if let Ok(mut parsed) = pg_query::parse(&message.query) {
                    let mut modified = false;
                    unsafe {
                        for (node, _depth, _context) in parsed.protobuf.nodes_mut().into_iter() {
                            modified = self.transform_stmt(node, username) || modified;
                        }
                    }
                    if modified {
                        let new_query = parsed.deparse()?;
                        debug!("New query: {}", &new_query);
                        Ok(PostgresqlPacket {
                            info: PostgresqlPacketInfo::Query(QueryMessage::from_query(new_query)),
                            bytes: None,
                        })
                    } else {
                        Ok(packet.clone())
                    }
                } else {
                    Err(anyhow!("Couldn't parser query: {}", message.query))
                }
            } else {
                Err(anyhow!("Expected auth context to be set for query message: {}", message.query))
            }
        } else {
            Ok(packet.clone())
        }
    }
}