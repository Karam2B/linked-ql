pub trait TableExpressions<'a> {
    type PascalCase;
    type SnakeCase;
    fn pascal_case(&'a self) -> Self::PascalCase;
    fn snake_case(&'a self) -> Self::SnakeCase;
}

/// members identifiers, to implement Join<Identifiers>: Expression<'q, S>
///
/// example where to be expected:
///     - `INSERT INTO Table ($here) Values (..) RETURNING ($here)`
///     - `UPDATE Table (..) RETURNING $here`
///     - `DELETE FROM Table RETURNING $here`
///
/// In select statements and where clauses use `MembersAliasedExpressions` instead.
pub trait MembersExpressions<'a> {
    type Identifiers;

    fn identifiers(&'a self) -> Self::Identifiers;
}

/// members aliased, to implement Join<Scoped/Aliased/NumAliased>: Expression<'q, S>
///
/// example where to be expected:
///     - `SELECT ($here) FROM Table`
///
/// In returning clauses use `Identifiers` instead.
pub trait MembersAliasedExpressions<'a>: MembersExpressions<'a> {
    /// members scoped by their table name
    type Scoped;
    fn scoped(&'a self) -> Self::Scoped;

    /// members scoped by their table name and aliased as follows: `$alias_$member`
    type ScopedAliased;
    fn scoped_aliased(&'a self, alias: &'static str) -> Self::ScopedAliased;

    /// members scoped by their table name and aliased as follows: `$alias_$num_$member`
    /// this number is used when multiple links define the same alias
    type NumScopedAliased;
    fn num_scoped_aliased(
        &'a self,
        num: &'static usize,
        alias: &'static str,
    ) -> Self::NumScopedAliased;

    /// heap heavy members information: (table_name_snake_case, table_name_pascal_case, Vec<member>)
    fn members(&self) -> (String, String, Vec<String>);
}

/// bind data to sql in insert statements
/// used in `INSERT INTO Table ($identifiers) Values ($values), ($values)`
///
/// ## Associated Types
/// - InputData is user-facing type, expected to be passed to Operations
/// - both Join<InsertValues> and Join<InsertIdentifiers> are expected to implement Expression
pub trait OnInsert<'a> {
    type InputData;
    type InsertValues;
    type InsertIdentifiers;
    fn on_insert_values(&'a self, input: Self::InputData) -> Self::InsertValues;
    fn on_insert_identifiers(&'a self) -> Self::InsertIdentifiers;
}

pub trait OnUpdate<'a> {
    type InputData;
    type UpdateSets;
    fn on_update_sets(&'a self, input: Self::InputData) -> Self::UpdateSets;
}

pub trait MigrationExpression<'a> {
    type Migrate;
    fn migrate(&'a self) -> Self::Migrate;
}
