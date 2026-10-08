use sqlx::Database;

mod database_any_impls;
mod database_sqlite_impls;

pub trait DatabaseExt: Database + DataBaseExtForExpressions {}

pub trait DataBaseExtForExpressions {
    type StatementBuilderInfo;
}

pub trait InMemoryConnection {
    fn in_memory_connection() -> impl Future<Output = <Self as sqlx::Database>::Connection> + Send;
    fn in_memory_pool() -> impl Future<Output = Pool<Self>>;
}
