use super::*;

#[derive(Debug, Clone, From)]
pub(super) struct UserObject {
    pub record: Record<User>,
}

#[Object(name = "User")]
impl UserObject {
    async fn id(&self) -> Id<User> {
        self.record.id().into()
    }

    async fn created_at(&self) -> DateTimeScalar {
        self.record.created_at().into()
    }

    async fn updated_at(&self) -> DateTimeScalar {
        self.record.updated_at().into()
    }

    async fn name(&self) -> &str {
        &self.record.name
    }

    async fn email(&self) -> &str {
        self.record.email.as_str()
    }

    async fn phone(&self) -> &str {
        self.record.phone.as_str()
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct UserQuery;

#[Object]
impl UserQuery {
    async fn user(
        &self,
        ctx: &Context<'_>,
        id: Id<User>,
    ) -> FieldResult<Option<UserObject>> {
        let services = ctx.services();
        let ctx = EntityContext::new(services);

        let user = User::get(id.into())
            .optional()
            .load(&ctx)
            .await
            .context("failed to load user")
            .into_field_result()?;
        let user = user.map(UserObject::from);
        Ok(user)
    }
}
