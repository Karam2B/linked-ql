pub mod basic_expressions;
pub mod combinators;
pub mod sanitize_combinator;
pub use basic_expressions::Bind;
pub use combinators::{Join, Prefixed};
pub use sanitize_combinator::Sanitize;
pub mod statement_builder;
pub mod statements;
pub mod std_impls;
pub mod trait_objects;
pub use statement_builder::*;

#[cfg(feature = "refactor")]
/// refactoring todos
/// - renaming OpExpression to SealExpression
/// - renameing RefOpExpression to SealRefExpression
/// - removing is_expression_present from SealExpression/OpExpression
/// - comments of this module should be the comments for sqlx_query_builder/mod.rs
mod refactor {
    /// Expression
    use super::database_extention::DatabaseExt;
    use super::statement_builder::StatementBuilder;

    /// Read module documentation [sqlx_query_builder/mod.rs](crate::sqlx_query_builder::mod.rs) for more information
    pub trait SealExpression {}

    /// Read module documentation [sqlx_query_builder/mod.rs](crate::sqlx_query_builder::mod.rs) for more information
    pub trait SealRefExpression: SealExpression {}

    /// Read module documentation [sqlx_query_builder/mod.rs](crate::sqlx_query_builder::mod.rs) for more information
    pub trait OptionalExpression {
        fn is_expression_present(&self) -> bool;
    }

    /// Read module documentation [sqlx_query_builder/mod.rs](crate::sqlx_query_builder::mod.rs) for more information
    pub trait RefExpression<'a, S>: SealRefExpression
    where
        S: DatabaseExt,
    {
        fn ref_expression<'q>(&'a self, ctx: &mut StatementBuilder<'q, S>);

        fn ref_sql_statement(&self) -> String {
            let mut this = StatementBuilder::default();
            self.ref_expression(&mut sb);
            if sb.count == 0 {
                return sb.stmt;
            } else {
                panic!("bug: any calls to ref_expression should not increment count")
            }
        }
    }

    pub trait Expression<'q, S>: SealExpression {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt;
        fn sql_statement(self) -> (String, S::Argument<'q>) {
            let mut sb = StatementBuilder::default();
            self.expression(&mut sb);
            sb.unwrap()
        }
        fn sql_statement_no_data(self) -> Option<String> {
            let mut this = StatementBuilder::default();
            self.expression(&mut sb);
            if sb.count == 0 {
                Some(sb.stmt);
            } else {
                None
            }
        }
    }

    impl<'q, S, T> Expression<'q, S> for T
    where
        S: DatabaseExt,
        T: for<'a> RefExpression<'a, S>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            self.ref_expression(ctx);
        }
    }
}

#[cfg(not(feature = "refactor"))]
mod stable {
    use super::statement_builder::StatementBuilder;
    use crate::database_extention::DatabaseExt;

    // impl<'q, S, T> Expression<'q, S> for T
    // where
    //     S: DatabaseExt,
    //     T: RefExpression<S>,
    // {
    //     fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
    //         self.ref_expression(ctx);
    //     }
    // }

    /// trait to lock implementing Expression over downstream's S
    ///
    /// Independant from S in Expression<'q, S>
    ///
    /// ## deprication
    /// is_op used to be a part of the trait, but no longer
    pub trait OpExpression {
        /// False when this value writes no SQL, so [`Join`](combinators::Join) can skip it.
        fn is_expression_present(&self) -> bool {
            true
        }
    }
    pub trait RefOpExpression: OpExpression {}

    pub trait RefExpression<'a, S>: RefOpExpression
    where
        S: DatabaseExt,
    {
        fn ref_expression<'q>(&'a self, ctx: &mut StatementBuilder<'q, S>);
    }

    /// Representing an sql string/expression/statement.
    ///
    /// # Type Generics `S`
    /// representing types that implement `sqlx::Database` like `Sqlite` and `MySQL`
    ///
    /// # lifetime Generics `'q`
    /// represent the ability to send a reference to an in-memory database
    ///
    /// example
    /// ```ignore
    ///     use sqlx::Sqlite;
    ///     use linked_sql::{
    ///         connect_in_memory::ConnectInMemory,
    ///         query_builder::{Expression, OpExpression, QueryBuilder},
    ///         use_executor,
    ///     };
    ///
    ///     struct Str<'a>(&'a str);
    ///     impl<'q> OpExpression for Str<'q> {}
    ///     impl<'q> Expression<'q, Sqlite> for Str<'q> {
    ///         fn expression(self, ctx: &mut QueryBuilder<'q, Sqlite>) {
    ///             ctx.syntax(&"SELECT ");
    ///             ctx.bind(self.0);
    ///             ctx.syntax(&";");
    ///         }
    ///     }
    ///
    ///     #[tokio::main]
    ///     async fn main() {
    ///         let pool = Sqlite::connect_in_memory().await;
    ///
    ///         let mut statment = String::from("hello world");
    ///
    ///         let holding_lifetime = QueryBuilder::new(Str(
    ///             &   /*'statment*/   statment
    ///         ));
    ///         
    ///         // restricted region
    ///         // let _ = &mut statment
    ///
    ///         let lifetime_droped = use_executor!(fetch_one(&pool, holding_lifetime)).unwrap();
    ///     }
    /// ```
    ///
    /// the only reason why there is lifetime in expression interface is: because in Sqlite you can send a string reference to an in-memory-database (instead of serializing the ref to an owned String and sending it over the netword like MySQL and PostgreQL), so you would have to wait for the lifetime (impl Expression<Sqlite, 'q> for &'q str) to be droped before you can mutate or move the referenced string
    ///
    /// in fact all impelentation of `sqlx::Encode` (used internally by `QueryBuilder::bind`) are static expect for `impl<'s> Encode<'s, Sqlite> for &'s str`
    ///
    /// this lifetime itroduce restriction on all references made between `holding_lifetime` and `lifetime_droped`
    ///
    /// Don't feel the need to abstract over this lifetime, look for statics instead of introducing a new lifetime (i.e. where T: Expression<'static, S>), especially when you are building over-the-netword backends where everything is static anyway, lifetime here is to have a perfect API that I don't need to refactor later.
    ///
    /// # Implementing `Expression`
    ///
    /// ## Composition
    /// Types that implement `Expression` often are composable, there are only
    /// two valid ways to compose generics.
    ///
    /// 1. `where Generic: Expression<'q, S>`
    ///
    /// ```ignore
    /// struct ColEq<C, V> {
    ///     pub col: C,
    ///     pub value: V,
    /// }
    /// impl<'q, S, C, V> Expression<'q, S> for ColEq<C, V>
    /// where
    ///     C: Expression<'q, S>,
    ///     V: Expression<'q, S>,
    /// {
    ///     ... some code ...
    /// }
    /// ```
    ///
    /// note that this put the responsibility of valid composition on the creator of the type,
    /// this is valid `ColEq { col: "column", value: Bind(34) }` while
    /// this is invalid `ColEq { col: "column", value: 34 }`, because i32
    /// by itself does not implement `Expression`.
    ///
    /// 2. `where Join<Generic>: Expression<'q, S>`
    /// this is where you need to join multple `impl Expression`s together,
    ///
    /// ## Non-operational implementation
    /// all implementation should be operation -- meaning they add some content to the sql
    /// statement, meaning types like (), Option<T>, Vec<T> should not implement Expression.
    ///
    /// The only exception is `Join` and `Prefixed`, they allow these optional types
    /// to be used
    ///
    /// ## `Join`` type
    ///
    /// important type you should be aware of is `Join`, it implements `Expression` when its `Item` generic represents
    /// multiple `Expression`s.
    /// ```ignore
    /// fn test() {
    ///     let join = Join {
    ///         start: "SELECT ",
    ///         separator: ", ",
    ///         items: (
    ///             "column",
    ///             ColumnAs { column: Bind(34), as_: "new_column" }
    ///         )
    ///     };
    ///     
    ///     assert_eq!(join.string_only(), "SELECT column, $1 AS new_column");
    /// }
    /// ```
    ///
    /// `Join` is the only type that can be used in generic composition. It
    /// should not be directly used by consumer, but rather internally by
    /// implementors of `Expression`.
    ///
    /// ## `Bind` type
    /// Constrain generics by `'q` is only for `Bind` type
    ///
    /// ```ignore
    /// impl<'q, S, V> Expression<'q, S> for Bind<V>
    /// where
    ///     V: 'q + Encode<'q, S>,
    /// ```
    ///
    /// other types rely on composition, for example
    ///
    /// ```ignore
    /// struct ColEq<C, V> {
    ///     pub col: C,
    ///     pub value: V,
    /// }
    /// impl<'q, S, C, V> Expression<'q, S> for ColEq<C, V>
    /// where
    ///     C: Expression<'q, S>,
    ///     V: Expression<'q, S>,
    /// { ... some code ... }
    ///
    /// fn valid_composition() {
    ///     let col_eq = ColEq {
    ///         col: "column",
    ///         value: Bind(34),
    ///     };
    ///     
    ///     assert_eq!(col_eq.string_only(), "column = $1");
    /// }
    /// ```
    ///
    /// ## lifetime 'q
    /// This lifetime represent the ability to send a reference to an in-memory database.
    ///
    /// In generic composition, 'q is used like this:
    /// `where Generic: Expression<'q, S>`
    ///
    /// Generic should never be constrained by `'q`, like:
    /// `where Generic: 'q + Expression<'q, S>`,
    /// the only exception for `Bind`'s Generic.
    pub trait Expression<'q, S>: OpExpression {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt;
    }

    impl<'q, S, T> Expression<'q, S> for T
    where
        S: DatabaseExt,
        T: for<'a> RefExpression<'a, S>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            self.ref_expression(ctx);
        }
    }
}

#[cfg(feature = "refactor")]
pub use refactor::*;
#[cfg(not(feature = "refactor"))]
pub use stable::*;

#[cfg(test)]
mod fix_lifetime_tests {

    use crate::sqlx_query_builder::{
        StatementBuilder, statements::select_statement::SelectStatement,
    };
    use sqlx::Sqlite;

    #[test]
    fn main() {
        let (stmt, _args) = StatementBuilder::<Sqlite>::new(SelectStatement {
            select_items: ("1",),
            from: "t",
            joins: (),
            wheres: (),
            group_by: (),
            order: (),
            limit: (),
        })
        .unwrap();

        assert_eq!(stmt, r#"SELECT "1" FROM "t";"#);
    }
}
