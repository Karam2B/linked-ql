pub mod basic_expressions;
pub mod combinators;
#[cfg(not(feature = "refactor"))]
pub mod sanitize_combinator;
#[cfg(feature = "refactor")]
/// refactoring todos
/// - use this mod at the expense of sanitize_combinator and std_impls
pub mod sanitize_impls;
pub use basic_expressions::Bind;
pub use combinators::{Join, Prefixed};
pub use sanitize_combinator::Sanitize;
/// refactoring todos
/// - change StatementBuilder fields
/// - work closely with DatabaseStatementBuilder trait
/// - StatementBuilder::bind is replaced by internal calls on Bind type
/// - StatementBuilder::sanitize is replaced by internal calls on strings and Sanitize type
/// - StatementBuilder::type_as_syntax is replaced by internal calls on TypeAsSyntax type
/// - unwrap is replaced by default implementation for *Expression::*sql_statement*
#[cfg(not(feature = "refactor"))]
pub mod statement_builder;
#[cfg(feature = "refactor")]
#[path = "statement_builder_v2.rs"]
pub mod statement_builder;
#[cfg(feature = "refactor")]
#[linked_sql_macros::skip]
/// skiped to work on compiling errors incrementally
pub mod statements;
#[cfg(not(feature = "refactor"))]
pub mod statements;
#[cfg(not(feature = "refactor"))]
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
    //! # Composible traits to write SQL statements
    //!
    //! ## Composibility
    //! implementations of Expression should be genericly composible, like this:
    //! ```no_run
    //! use crate::sqlx_query_builder::{Expression, StatementBuilder};
    //!
    //! struct ColumnEquals<C, V> {
    //!     pub col: C,
    //!     pub value: V,
    //! }
    //!
    //! impl<'q, C, V> Expression<'q, S> for ColumnEquals<C, V>
    //! where
    //!     C: Expression<'q, Sqlite>,
    //!     V: Expression<'q, Sqlite>,
    //! {
    //!     ... some code ...
    //! }
    //!
    //! ```
    //!
    //! ## Wrong composition
    //! It is possible to have problems with wrong composition, consider this example:
    //! ```no_run
    //! fn test() {
    //!     let wrong_col_eq = ColumnEquals {
    //!         col: ColumnEquals {
    //!             col: "column",
    //!             value: Bind(34),
    //!         },
    //!         value: Bind(34),
    //!     }; // `column = column = $1` is invalid SQL
    //!     panic!("no compile errors, but incorrect composition: {}", wrong_col_eq.sql_statement());
    //! }
    //! ```
    //!
    //! Traits that require correct compositions lives in 'crate::valid_syntax' module
    //!
    //! ## Type Generic 'S'
    //! Types that represent different sql dialects, like `Sqlite`, `MySQL`, `Postgres`, etc.
    //! these types are `impl sqlx::Database`
    //!
    //!
    //! ## When to specify 'S' and when to leave it implicit?
    //! Some databases expect different syntax as others, in this case you should specify 'S' explicitly.
    //!
    //! Even in cases where that is not the case, it is recommended to specify 'S' explicitly, to open the door
    //! for future database support.
    //!
    //! In case of being generic over 'S', you have to add this constraint:
    //! `impl<S: crate::extend_sqlx::DatabaseStatementBuilder> ... the rest ...`
    //!
    //!
    //! ## Lifetime Generic 'q'
    //! This is the same lifetime in `sqlx::Arguments<'q>`.
    //!
    //! As I understand it, it's used to represent the ability to send a reference to an in-memory database.
    //! Databases like `Sqlite` provide the following impls:
    //! ```rust
    //! impl<'q> sqlx::Encode<'q, Sqlite> for &'q str {
    //!     ... send a reference to an in-memory database ...
    //! }
    //!
    //! impl<'q> sqlx::Decode<'q, Sqlite> for &'q str {
    //!     ... receive a string reference from an in-memory database ...
    //! }
    //! ```
    //!
    //! As far as I know there is no other use of this lifetime except this impl. Everything else can be
    //! assumed to be static.
    //!
    //! ## using 'q in implementing generics
    //! in composed generics you should pass 'q to the inner Expression, never constraint
    //! the generic itself by 'q. Example:
    //!
    //! ```no_run
    //! impl<'q, S, C, V> Expression<'q, S> for ColumnEquals<C, V>
    //! where
    //!     C: Expression<'q, S>,
    //!     V: Expression<'q, S>,
    //! {
    //!     ... some code ...
    //! }
    //! ```
    //!
    //! The incorrect way to do it is:
    //! ```no_run
    //! impl<'q, S, C, V> Expression<'q, S> for ColumnEquals<C, V>
    //! where
    //!     C: 'q + Expression<'q, S>,
    //!     V: 'q + Expression<'q, S>,
    //! {
    //!     ... some code ...
    //! }
    //! ```
    //!
    //! Only `Bind` (which is implemented already) can constraint by 'q.
    //!
    //! ## What does 'q restrict?
    //! To visualize what 'q restricts, consider this example:
    //! ```no_run
    //!     #[tokio::main]
    //!     async fn main() {
    //!         let pool = Sqlite::connect_in_memory().await;
    //!
    //!         let mut column_name = String::from("column_name");
    //!
    //!         // lifetime is created here
    //!         // holding_lifetime types is: ColumnEquals<&'column_name str, Bind<i32>>
    //!         let holding_lifetime = ColumnEquals {
    //!             col: column_name.as_str(),
    //!             value: Bind(34),
    //!         };
    //!
    //!         // ============ Restricted region ============
    //!         // from the point `as_str` created a lifetime,
    //!         // to the point the lifetime is droped (`use_executor`
    //!         // took ownership of `holding_lifetime` and dropped it)
    //!         //
    //!         // Here, you cannot mutate or move `column_name` anymore
    //!         // code like this will not compile:
    //!         // `column_name.push_str("new_column_name");`
    //!         // ===========================================
    //!
    //!         // lifetime is droped here
    //!         use_executor!(fetch_one(&pool, holding_lifetime)).unwrap();
    //!     }
    //! ```
    //!
    //! ## Bind Type
    //! Special type that allows you to bind a value to a statement, its Expression
    //! implementation follows sqlx constraints. As follows:
    //! ```no_run
    //! impl<'q, S, V> Expression<'q, S> for Bind<V>
    //! where
    //!     V: 'q + Encode<'q, S>,
    //! {
    //!     ... some code ...
    //! }
    //!
    //! fn use_bind() {
    //!     let bind = Bind(34);
    //!     let (stmt, args) = bind.sql_statement();
    //!     assert_eq!(stmt, "$1");
    //! }
    //!
    //! ```
    //!
    //! ## Sanitization
    //! All standard library string types are sanitized, in addition to `Sanitize` and `ArcSubStr`
    //!
    //! `Sanitize` type supports joining multiple items and sanitize them. Example:
    //! ```no_run
    //! fn test() {
    //!     let sanitize = Sanitize(("column_name_", 32, "_sufix"));
    //!     assert_eq!(sanitize.sql_statement(), "\"column_name_32_sufix\"");
    //! }
    //! ```
    //!
    //! ## Join and Prefixed types
    //! `Join` and `Prefixed` are a special types that allows you to join multiple `Expression`s together. Read
    //! [combinators::Join](combinators::Join) for more information.
    //!
    //! ## Sealing
    //! traits like `SealExpression` and `SealRefExpression` are used to seal the trait over downstream's S.
    //! This is usefull in blanket implementations
    //!
    //! ## passing by ref vs passing by value
    //! `Expression` trait takes `self`, while `RefExpression` takes `&self`.
    //! types that implement `Expression` usually will contain a `Bind` down the chain,
    //! where as types that implement `RefExpression` only provide information to the SQL string, not buffer.
    //!
    //! ## Empty Expressions
    //! Most types that implement `Expression` or `RefExpression` should write some SQL.
    //! to use possbily empty types like `Option<T>`, `Vec<T>`
    //! consider using Join or Prefix. As they do not implement `Expression` or `RefExpression`.
    //! Example:
    //! ```no_run
    //! fn test() {
    //!     let string = Join(("KEYWORD ", Option::<String>::None, Some("VALUE")));
    //!     assert_eq!(string.sql_statement(), "KEYWORD VALUE");
    //! }
    //! ```
    //!
    //! Join type report on whether it will write any SQL by `OptionalExpression` trait.
    //! Usually you don't need to implement this trait for types that implement `Expression` or `RefExpression`.
    //!
    //! ## Protection against SQL injection
    //! Most area of accidental SQL injection are protected by either sanitization or binding.
    //! All these types do not introduce SQL injection: "String::new("malicious_input")", "Bind(34)", .
    //!
    //! Even `StatementBuilder::syntax` require `'static` constraints, so the only way to introduce SQL injection is
    //! like this:
    //!
    //! ```run
    //! impl<'q, S> Expression<'q, S> for MaliciousType {
    //!     fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
    //!         // 'static requirement provide protection against SQL injection
    //!         // to introduce SQL injection you have to write an obvious malicious code like this:
    //!         let get_static_str = Box::from(self.string_from_network.as_str());
    //!         let leak = Box::leak(get_static_str);
    //!         ctx.syntax(leak);
    //!     }
    //! }
    //! ```
    //!

    use sqlx::Database;

    use super::statement_builder::StatementBuilder;
    use crate::database_extention::DatabaseExt;

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

        fn ref_sql_statement(&self) -> String
        where
            S: DatabaseExt,
        {
            let mut sb = StatementBuilder::default();
            self.ref_expression(&mut sb);
            if S::is_buffer_empty(&sb) {
                sb.stmt
            } else {
                panic!("bug: any calls to ref_expression should not increment count")
            }
        }
    }

    pub trait Expression<'q, S>: SealExpression {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>)
        where
            S: DatabaseExt;

        fn sql_statement(self) -> (String, <S as Database>::Arguments<'q>)
        where
            Self: Sized,
            S: Database,
        {
            let mut sb = StatementBuilder::default();
            self.expression(&mut sb);
            (sb.stmt, sb.arg)
        }

        fn sql_statement_no_data(self) -> Option<String>
        where
            Self: Sized,
            S: DatabaseExt,
        {
            let mut sb = StatementBuilder::default();
            self.expression(&mut sb);
            if S::is_buffer_empty(&sb) {
                Some(sb.stmt)
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
#[cfg(feature = "refactor")]
mod refactor_fix_lifetime_tests {
    use crate::sqlx_query_builder::{Expression, statements::select_statement::SelectStatement};
    use sqlx::Sqlite;

    #[test]
    fn main() {
        let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(SelectStatement {
            select_items: ("1",),
            from: "t",
            joins: (),
            wheres: (),
            group_by: (),
            order: (),
            limit: (),
        });

        assert_eq!(stmt, r#"SELECT "1" FROM "t";"#);
    }
}

#[cfg(test)]
#[cfg(not(feature = "refactor"))]
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
