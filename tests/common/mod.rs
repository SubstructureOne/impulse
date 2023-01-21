use std::collections::HashMap;

use anyhow::Result;
// use async_once::AsyncOnce;
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, embed_migrations, MigrationHarness};
use dotenvy::dotenv;
use log::{info};
use lazy_static::lazy_static;

use std::env;
use std::rc::Rc;
use chrono::{DateTime, Duration};
use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;
// use docker_api::{Container, Docker};
// use docker_api::opts::{ContainerCreateOpts, PublishPort};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
pub const DB_PREFIX: &str = "ImpulseTestingDb_";

lazy_static! {
    pub static ref ENV: HashMap<String, String> = {
        dotenv().ok();
        let _ = env_logger::builder().is_test(true).try_init();
        HashMap::from_iter(env::vars())
    };
    // pub static ref BASE_URL: String = {
    //     dotenv().ok();
    //     let _ = env_logger::builder().is_test(true).try_init();
    //     ENV.get("TESTING_BASE_URL").expect("Must specify TESTING_BASE_URL").clone()
    // };
    // static ref PG_CONTAINER: AsyncOnce<PostgresContainer> = AsyncOnce::new(async {
    //     PostgresContainer::start().await.expect("Couldn't initialize docker container")
    // });
}

// struct PostgresContainer {
//     container: Container,
//     base_url: String,
// }
// impl PostgresContainer {
//     pub async fn start() -> Result<PostgresContainer> {
//         let docker = Docker::unix("/var/run/docker.sock");
//         let env = vec!["POSTGRES_PASSWORD=pw"];
//         let opts = ContainerCreateOpts::builder()
//             .image("postgres:15")
//             .name("postgres")
//             .expose(PublishPort::tcp(5432), 9432)
//             .env(&env)
//             .build();
//         let container = docker.containers().create(&opts).await?;
//         container.start().await?;
//         Ok(
//             PostgresContainer {
//                 container,
//                 base_url: "postgres://postgres:pw@localhost:9432".to_string(),
//             }
//         )
//     }
// }

pub struct TestContext {
    pub impulse_config: Rc<ManagementConfig>,
    pub impulse_manager: PostgresManager,
    pub managed_db_manager: PostgresManager,
    pub db_name: String,
}

impl TestContext {
    pub fn new(db_name: &str) -> Result<Self> {
        // let postgres_url = &PG_CONTAINER.get().await.base_url;
        let impulse_config = Rc::new(ManagementConfig::new(
            ENV.get("TESTING_DB_HOST").expect("Must specify TESTING_DB_HOST"),
            ENV.get("TESTING_DB_PORT").expect("Must specify TESTING_DB_PORT").parse::<u32>()?,
            ENV.get("TESTING_DB_USER").expect("Must specify TESTING_DB_USER"),
            ENV.get("TESTING_DB_PASSWORD").expect("Must specify TESTING_DB_PASSWORD"),
        ));
        let impulse_manager = PostgresManager::new(impulse_config.clone());
        info!("Testing environment (impulse database): {}", &impulse_manager);
        let managed_db_config = Rc::new(ManagementConfig::new(
            ENV.get("MANAGED_DB_HOST").expect("Must specify MANAGED_DB_HOST"),
            ENV.get("MANAGED_DB_PORT").expect("Must specify MANAGED_DB_PORT").parse::<u32>()?,
            ENV.get("MANAGED_DB_USER").expect("Must specify MANAGED_DB_USER"),
            ENV.get("MANAGED_DB_PASSWORD").expect("Must specify MANAGED_DB_PASSWORD"),
        ));
        let managed_db_manager = PostgresManager::new(managed_db_config.clone());
        info!("Testing environment (managed database): {}", &managed_db_manager);

        let mut impulse_conn = impulse_manager.pg_connect()?;
        let db_name = DB_PREFIX.to_string() + db_name;
        info!("Creating database {}", db_name);
        let query = diesel::sql_query(format!(r#"CREATE DATABASE "{}""#, &db_name));
        query.execute(&mut impulse_conn)?;

        info!("Running migrations on {}", db_name);
        let mut conn = impulse_manager.pg_connect_db(&db_name)?;
        conn.run_pending_migrations(MIGRATIONS).unwrap();

        Ok(
            TestContext {
                impulse_config,
                impulse_manager,
                managed_db_manager,
                db_name,
            }
        )
    }

    fn drop_database(&self) -> Result<()> {
        // any changes made to the managed database are expected to be rolled
        // back by the test itself.
        self.impulse_manager.drop_database(&self.db_name)?;
        Ok(())
    }
}


impl Drop for TestContext {
    fn drop(&mut self) {
        match self.drop_database() {
            Ok(()) => {}
            Err(err) => {
                panic!("Couldn't drop testing database: {}.", err);
            }
        }
    }
}

/// Implement a "soft equality" between two items.
///
/// Used for testing that a result from, e.g., a database, effectively matches
/// the expected value. Separate from PartialEq/Eq because there might be
/// elements (such as a unique identifier created by the database) that we
/// don't know a priori and don't care about, or that are acceptable within a
/// given tolerance, like DateTimes.
pub trait ExpectedEquals {
    fn expected_equals(&self, other: &Self) -> bool;
}

impl ExpectedEquals for DateTime<chrono::Utc> {
    fn expected_equals(&self, other: &Self) -> bool {
        let duration = other.signed_duration_since(self.clone());
        // no Duration::abs?
        duration < Duration::minutes(1) && -duration < Duration::minutes(1)
    }
}
