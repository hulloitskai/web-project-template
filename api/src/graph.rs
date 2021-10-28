mod query;
pub use query::*;

// mod mutation;
// pub use mutation::*;

mod build;
use build::*;

// mod date;
// use date::*

mod date_time;
use date_time::*;

mod id;
use id::*;

mod user;
use user::*;

use entrust::{Comparison, Record, SortingDirection};
use entrust::{Entity, EntityId};

use graphql::scalar;
use graphql::Context;
use graphql::SimpleObject;
use graphql::Value;
use graphql::{Enum, EnumType};
use graphql::{FieldError, FieldResult};
use graphql::{InputObject, InputObjectType};
use graphql::{InputValueError, InputValueResult};
use graphql::{Interface, InterfaceType};
use graphql::{MergedObject, Object, ObjectType};
use graphql::{MergedSubscription, Subscription, SubscriptionType};
use graphql::{Scalar, ScalarType};
use graphql::{Union, UnionType};

use super::*;

use entities::{Context as EntityContext, *};
use services::Services;

#[async_trait]
pub(super) trait ContextExt {
    fn services(&self) -> Services;

    async fn transact<F, T, U>(&self, f: F) -> FieldResult<T>
    where
        F: Send,
        F: FnOnce(EntityContext) -> U,
        T: Send,
        U: Send,
        U: Future<Output = Result<T>>,
    {
        let services = self.services();
        let ctx = EntityContext::new(services);
        ctx.transact(f).await.into_field_result()
    }
}

impl<'a> ContextExt for Context<'a> {
    fn services(&self) -> Services {
        let services = self.data_unchecked::<Services>();
        services.to_owned()
    }
}

pub(super) trait ResultExt<T> {
    fn into_field_result(self) -> FieldResult<T>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    Result<T, E>: AnyhowContext<T, E>,
    E: Display,
{
    fn into_field_result(self) -> FieldResult<T> {
        self.map_err(|error| FieldError::new(format!("{:#}", error)))
    }
}
