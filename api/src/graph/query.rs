use super::*;

#[derive(Debug, Clone, Copy, MergedObject)]
pub struct Query(BuildQueries, UserQueries);

impl Query {
    pub fn new() -> Self {
        Self(BuildQueries, UserQueries)
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new()
    }
}
