use sqlx::{Database, Pool};
use sqlx::{Encode, Type};
mod database_any_impls;
mod database_sqlite_impls;

pub trait DatabaseExt: Database + DatabaseStatementBuilder {}

/// Trait that handles how each database writes SQL statements
pub trait DatabaseStatementBuilder: Database {
    /// used as a field in StatementBuilder, usually it is just a count: usize
    type StatementBuilderInfo: Default;
    /// used internally by Bind type
    fn impl_bind<'q, V>(
        value: V,
        stmt: &mut String,
        info: &mut Self::StatementBuilderInfo,
        arg: &mut Self::Arguments<'q>,
    ) where
        V: Encode<'q, Self> + 'q + Type<Self>;

    /// used internally by *Expression::*sql_statement*
    fn is_buffer_empty(info: &Self::StatementBuilderInfo) -> bool;

    /// used internally by TypeAsSyntax type
    fn type_as_syntax<T: Type<Self>>(into: &mut String);

    /// used internally by strings, Sanitize types
    fn sanitize_start(into: &mut String);

    /// used internally by strings, Sanitize types
    fn sanitize_end(into: &mut String);

    /// used internally by strings, Sanitize types
    fn sanitize_char(char: char, into: &mut String);

    /// used internally by strings, Sanitize types
    fn sanitize_segment(string: &str, into: &mut String);
}

/// Trait that specify important `impl Expression` types
/// used in implementing Operation and valid_syntax traits
pub trait DatabaseExpressions {}

/// Database that supports in-memory connections
/// usually only sqlite supports this
pub trait InMemoryConnection: Database {
    fn in_memory_connection() -> impl Future<Output = Self::Connection> + Send;
    fn in_memory_pool() -> impl Future<Output = Pool<Self>>;
}
