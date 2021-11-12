use super::*;

#[derive(Debug, Clone, Copy, MergedObject)]
pub struct Mutation(TestMutation);

impl Mutation {
    pub fn new() -> Self {
        Self(TestMutation)
    }
}

impl Default for Mutation {
    fn default() -> Self {
        Self::new()
    }
}
