#![allow(dead_code, unused_imports)]

mod build;
pub use build::*;

mod email;
pub use email::*;

mod phone;
pub use phone::*;

mod user;
pub use user::*;

use super::*;

use entrust::{AggregateOneQuery, AggregateQuery, MaybeAggregateOneQuery};
use entrust::{Comparison, SortingDirection};
use entrust::{Database, DatabaseClient};
use entrust::{EmptyConditions, EntityConditions};
use entrust::{EmptySorting, EntitySorting};
use entrust::{Entity, EntityContext, EntityId, EntityServices};
use entrust::{FindOneQuery, FindQuery, MaybeFindOneQuery};
use entrust::{Object, ObjectId};

use ::bson::DateTime as BsonDateTime;
use ::bson::{doc, from_document, to_document};
use ::bson::{Bson, Document};

fn to_date_time(date: Date) -> DateTime {
    let time = Time::from_hms(0, 0, 0);
    let date_time = date.and_time(time);
    Utc.from_utc_datetime(&date_time)
}

fn from_date_time(date_time: DateTime) -> Date {
    date_time.naive_utc().date()
}

#[derive(Debug, Builder)]
pub struct Services {
    database: Database,
    database_client: DatabaseClient,
    settings: Settings,
}

#[derive(Debug, Clone, Builder)]
pub struct Settings {
    pub web_public_url: Url,
    pub api_public_url: Url,
}

impl Services {
    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn database_client(&self) -> &DatabaseClient {
        &self.database_client
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
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

pub type Context<T = Services> = EntityContext<T>;
