use super::*;

#[derive(Debug, Clone, Copy, MergedObject)]
pub struct Query(BuildQuery, UserQuery);

impl Query {
    pub fn new() -> Self {
        Self(BuildQuery, UserQuery)
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new()
    }
}
