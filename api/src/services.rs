use super::*;

use entrust::EntityServices;
use entrust::{Database, DatabaseClient};

#[derive(Debug, Builder)]
pub struct Config {
    pub database: Database,
    pub database_client: DatabaseClient,
    pub settings: Settings,
}

#[derive(Debug)]
struct ServicesInner {
    database: Database,
    database_client: DatabaseClient,
    settings: Settings,
}

impl ServicesInner {
    fn database(&self) -> &Database {
        &self.database
    }

    fn database_client(&self) -> &DatabaseClient {
        &self.database_client
    }

    fn settings(&self) -> &Settings {
        &self.settings
    }
}

#[derive(Debug, Clone)]
pub struct Services(Arc<ServicesInner>);

impl Services {
    pub fn new(config: Config) -> Self {
        let Config {
            database,
            database_client,
            settings,
        } = config;

        let inner = ServicesInner {
            database,
            database_client,
            settings,
        };
        Services(inner.into())
    }

    delegate! {
        to self.0 {
            pub fn database(&self) -> &Database;
            pub fn database_client(&self) -> &DatabaseClient;
            pub fn settings(&self) -> &Settings;
        }
    }
}

impl EntityServices for Services {
    fn database(&self) -> &Database {
        self.database()
    }

    fn database_client(&self) -> &DatabaseClient {
        self.database_client()
    }
}

#[derive(Debug, Clone, Builder)]
pub struct Settings {
    pub api_url: Url,
    pub api_public_url: Url,
    pub web_url: Url,
    pub web_public_url: Url,
}
