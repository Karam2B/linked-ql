pub trait TableExpressions<'a> {
    type PascalCase;
    type SnakeCase;
    fn pascal_case(&'a self) -> Self::PascalCase;
    fn snake_case(&'a self) -> Self::SnakeCase;
}

pub trait MembersExpressions<'a> {
    type Identifier;
    fn identifier(&'a self) -> Self::Identifier;
}

pub trait MembersAliasedExpressions<'a>: MembersExpressions<'a> {
    type Scoped;
    fn scoped(&'a self) -> Self::Scoped;

    type ScopedAliased;
    fn scoped_aliased(&'a self, alias: &'static str) -> Self::ScopedAliased;

    type NumScopedAliased;
    fn num_scoped_aliased(&'a self, num: usize, alias: &'static str) -> Self::NumScopedAliased;

    fn members(&self) -> (String, Vec<String>);
}

pub trait OnInsertHandler<'a, Input> {
    type InsertValue;
    type InsertIdentifier;
    fn on_insert(&'a self, input: Input) -> (Self::InsertIdentifier, Self::InsertValue);
}

pub trait OnInsert {
    type InsertValue;
    type InsertIdentifier;
    fn on_insert(self) -> (Self::InsertIdentifier, Self::InsertValue);
}

pub trait OnUpdate<'a, Input> {
    type UpdateSets;
    fn on_update(&'a self, input: Input) -> Self::UpdateSets;
}

pub trait MigrationExpression<'a> {
    type Migrate;
    fn migrate(&'a self) -> Self::Migrate;
}
