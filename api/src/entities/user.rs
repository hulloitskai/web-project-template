use super::*;

pub type UserId = EntityId<User>;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Object)]
pub struct User {
    pub handle: Handle,
    pub name: String,
    pub email: Email,
    pub phone: Phone,
}

impl Entity for User {
    const NAME: &'static str = "User";

    type Services = Services;
    type Conditions = UserConditions;
    type Sorting = UserSorting;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
pub struct UserConditions {
    #[builder(default, setter(into))]
    pub email: Option<Email>,

    #[builder(default, setter(into))]
    pub phone: Option<Phone>,
}

impl EntityConditions for UserConditions {
    fn into_document(self) -> Document {
        let UserConditions { email, phone } = self;
        let mut doc = Document::new();

        if let Some(email) = email {
            doc.insert("email", email);
        }
        if let Some(phone) = phone {
            doc.insert("phone", phone);
        }

        doc
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserSorting {
    Name(SortingDirection),
}

impl EntitySorting for UserSorting {
    fn into_document(self) -> Document {
        use UserSorting::*;
        match self {
            Name(direction) => doc! { "name": direction },
        }
    }
}

impl User {
    pub fn find_by_email(email: Email) -> FindOneQuery<Self> {
        User::find_one(UserConditions::builder().email(email).build())
    }
}
