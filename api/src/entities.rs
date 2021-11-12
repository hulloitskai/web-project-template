#![allow(unused_imports)]

mod build;
mod email;
mod handle;
mod phone;
mod user;

pub use build::*;
pub use email::*;
pub use handle::*;
pub use phone::*;
pub use user::*;

use super::*;

use services::Services;

pub type Context<T = Services> = EntityContext<T>;

use entrust::Record;
use entrust::{AggregateOneQuery, AggregateQuery, MaybeAggregateOneQuery};
use entrust::{Comparison, SortingDirection};
use entrust::{EmptyConditions, EntityConditions};
use entrust::{EmptySorting, EntitySorting};
use entrust::{Entity, EntityContext, EntityId, EntityServices};
use entrust::{FindOneQuery, FindQuery, MaybeFindOneQuery};
use entrust::{Object, ObjectId};

use ::bson::DateTime as BsonDateTime;
use ::bson::{bson, doc, from_document, to_document};
use ::bson::{Bson, Document};
