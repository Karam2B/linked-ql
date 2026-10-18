#![allow(non_camel_case_types)]
//! Trait system for specifying the behavior of a link in an insert operation.
//!
//! # Insert Operation Flow
//! This flow is used internally by the `InsertOne` operation.
//!
//! ╭─────────────────────────────────────────╮
//! │       link as provided to operation     │
//! │                   │                     │
//! │    operation before insert statement    │
//! │                   │                     │
//! │      constructing insert statement      │
//! │                   │                     │
//! │    operation after insert statement     │
//! │                   │                     │
//! │              link output                │
//! ╰─────────────────────────────────────────╯
//!
//! # Data Ownership
//! In order to avoid unnecessary data cloning, there is only
//! one part `InsertLinkParts::LinkSpec` that is passed by reference.
//! everything else is passed by value.

// ====================================
// === Insert One Link Trait System ===
// ====================================
use crate::{from_row::FromRowData, operations::OperationOutput};

/// User facing types used in insert one operations.
/// Types implementing this trait are used directly by the user
/// to use a link in an insert one operation.
pub trait InsertOneLink<Base> {
    type LinkSpec: InsertOneLinkSpec;
    /// split expensive data to parts.
    fn init(
        self,
        base: Base,
    ) -> InsertParts<
        Self::LinkSpec,
        <Self::LinkSpec as InsertOneLinkSpec>::PreOperation,
        <Self::LinkSpec as InsertOneLinkSpec>::Init_InsertValues,
        <Self::LinkSpec as InsertOneLinkSpec>::Init_SqlStatement,
    >;
}

pub struct InsertParts<Spec, PreOperation, Init_InsertValues, Init_SqlStatement> {
    pub link_spec: Spec,
    pub pre_operation: PreOperation,
    pub init_insert_values: Init_InsertValues,
    pub init_sql_statement: Init_SqlStatement,
}

pub struct PreOperationParts<
    PreOperation_InsertValues,
    PreOperation_SqlStatement,
    PreOperation_Output,
> {
    pub pre_operation_insert_values: PreOperation_InsertValues,
    pub pre_operation_sql_statement: PreOperation_SqlStatement,
    pub pre_operation_output: PreOperation_Output,
}

pub struct SqlStatmentParts<PostOperation, SqlStatment_Output> {
    pub post_operation: PostOperation,
    pub sql_statement_output: SqlStatment_Output,
}

/// link implementation used internally in InsertOne
///
/// Data extracted by `Operation` or `FromRowAlias` is assumed
/// to be expensive and cloning is discouraged.
pub trait InsertOneLinkSpec {
    /// passed from `init` to insert_values methods
    type Init_InsertValues;
    /// passed from `init` to `sql_statement` method
    type Init_SqlStatement;

    /// operation executed before the main sql statement, to implement Operation<S>
    type PreOperation: OperationOutput;

    /// split PreOperation expensive data into parts
    fn pre_operation_parts(
        &self,
        pre_operation_output: <Self::PreOperation as OperationOutput>::Output,
    ) -> PreOperationParts<
        Self::PreOperation_InsertValues,
        Self::PreOperation_SqlStatement,
        Self::PreOperation_Output,
    >;

    /// passed from `pre_operation_parts` to `insert_values` method
    type PreOperation_InsertValues;
    /// passed from `pre_operation_parts` to `output` method
    type PreOperation_Output;
    /// passed from `pre_operation_parts` to `sql_statement` method
    type PreOperation_SqlStatement;

    /// part of the sql statement used in `INSERT INTO Table ($here) Values (..) RETURNING ($here)`
    fn insert_identifiers(&self) -> Self::InsertIdentifiers;
    /// part of sql statment, to implement Join<InsertIdentifiers>: Expression<'q, S>
    type InsertIdentifiers;

    /// part of the sql statement used in `INSERT INTO Table (..) Values ($here) RETURNING (..)`
    fn insert_values(
        &self,
        init_insert_values: Self::Init_InsertValues,
        pre_operation_insert_values: Self::PreOperation_InsertValues,
    ) -> Self::InsertValues;
    /// part of sql statment, to implement Join<InsertIdentifiers>: Expression<'q, S>
    type InsertValues;

    /// to retrieve members specified in RETURNING clause of the insert sql
    fn from_row(&self) -> Self::FromRow;
    /// to retrieve members specified in RETURNING clause of the insert sql, to implement FromRowAlias<'r, S::Row>
    type FromRow: FromRowData;

    /// split sql returned expensive data into parts
    fn sql_statement_parts(
        &self,
        init_sql_statement: Self::Init_SqlStatement,
        pre_operation_sql_statement: Self::PreOperation_SqlStatement,
        from_row_data: <Self::FromRow as FromRowData>::RData,
    ) -> SqlStatmentParts<Self::PostOperation, Self::SqlStatement_Output>;

    /// operation executed sql statement, to implement Operation<S>
    type PostOperation: OperationOutput;

    /// passed from `sql_statement` to `output`
    type SqlStatement_Output;

    /// to return link's output
    fn output(
        self,
        pre_operation_output: Self::PreOperation_Output,
        sql_statement_output: Self::SqlStatement_Output,
        post_operation_output: <Self::PostOperation as OperationOutput>::Output,
    ) -> Self::Output;

    /// main data extracted by the link
    type Output;
}

// ======================================
// === / Insert One Link Trait System ===
// ======================================

pub struct InsertOne<Handler, Attributes, Link, Infalibility> {
    pub handler: Handler,
    pub attributes: Attributes,
    pub link: Link,
    pub infalibility: Infalibility,
}

mod impl_operation_for_insert_one {
    use super::*;
    use crate::operations::{Operation, OperationOutput};
    use crate::operations_v2::fallibility::PanicOnFailure;
    use crate::sqlx_query_builder::statements::insert_statement::InsertStatement;
    use crate::{collections::Collection, operations::LinkedOutput};

    impl<Handler, Link, LinkSpec> OperationOutput
        for InsertOne<Handler, Handler::InputData, Link, PanicOnFailure>
    where
        Link: InsertOneLink<Handler, LinkSpec = LinkSpec>,
        LinkSpec: InsertOneLinkSpec,
        Handler: Collection,
    {
        type Output = LinkedOutput<Handler::OutputIdData, Handler::OutputData, LinkSpec::Output>;
    }

    impl<S, Handler, Link, LinkSpec> Operation<S>
        for InsertOne<Handler, Handler::InputData, Link, PanicOnFailure>
    where
        Link: InsertOneLink<Handler, LinkSpec = LinkSpec>,
        LinkSpec: InsertOneLinkSpec,
        Handler: Collection,
        Self: Send,
        Self::Output: Send,
    {
        fn exec_operation(self, _: &mut S::Connection) -> impl Future<Output = Self::Output> + Send
        where
            S: sqlx::Database,
            Self: Sized,
        {
            async move {
                let _ = InsertStatement {
                    table_name: (),
                    identifiers: (),
                    values: (),
                    returning: (),
                };
                todo!()
            }
        }
    }
}

pub mod insert_id_link {
    pub struct InsertId<T>(pub T);
}

mod impl_insert_one_system_for_many_links {
    use crate::operations_v2::insert_one::{InsertOneLink, InsertOneLinkSpec};

    macro_rules! impl_tuple {
        ($([$idx:literal $T:ty])*) => {

            #[allow(unused)]
            impl<Base, $($T,)*> InsertOneLink<Base> for ($($T,)*)
            where
                $(
                    $T: InsertOneLink<Base>,
                )*
            {
                type LinkSpec = ($(
                    $T::LinkSpec,
                )*);

                fn init(
                    self,
                    base: Base,
                ) -> super::InsertParts<
                    Self::LinkSpec,
                    <Self::LinkSpec as super::InsertOneLinkSpec>::PreOperation,
                    <Self::LinkSpec as super::InsertOneLinkSpec>::Init_InsertValues,
                    <Self::LinkSpec as super::InsertOneLinkSpec>::Init_SqlStatement,

                > {
                    let init_tuple = ($(
                        paste::paste!(self.$idx.init(base)),
                    )*);
                    super::InsertParts {
                        link_spec: ($(
                            paste::paste!(init_tuple.$idx.link_spec),
                        )*),
                        pre_operation: ($(
                            paste::paste!(init_tuple.$idx.pre_operation),
                        )*),
                        init_insert_values: ($(
                            paste::paste!(init_tuple.$idx.init_insert_values),
                        )*),
                        init_sql_statement: ($(
                            paste::paste!(init_tuple.$idx.init_sql_statement),
                        )*),

                    }
                }
            }


            #[allow(unused)]
            impl<$($T,)*> InsertOneLinkSpec for ($($T,)*)
            where $(
                $T: InsertOneLinkSpec,
            )*
            {
                type Init_InsertValues = ($(
                    $T::Init_InsertValues,
                )*);

                type Init_SqlStatement = ($(
                    $T::Init_SqlStatement,
                )*);

                type PreOperation = ($(
                    $T::PreOperation,
                )*);

                fn pre_operation_parts(
                    &self,
                    pre_operation_output: <Self::PreOperation as crate::operations::OperationOutput>::Output,
                ) -> super::PreOperationParts<
                    Self::PreOperation_InsertValues,
                    Self::PreOperation_SqlStatement,
                    Self::PreOperation_Output,
                > {
                    let tuple = ($(
                        paste::paste!(self.$idx.pre_operation_parts(
                            pre_operation_output.$idx,
                        )),
                    )*);
                    super::PreOperationParts {
                        pre_operation_insert_values: ($(
                            paste::paste!(tuple.$idx.pre_operation_insert_values),
                        )*),
                        pre_operation_output: ($(
                            paste::paste!(tuple.$idx.pre_operation_output),
                        )*),
                        pre_operation_sql_statement: ($(
                            paste::paste!(tuple.$idx.pre_operation_sql_statement),
                        )*),
                    }
                }

                type PreOperation_InsertValues = ($(
                    $T::PreOperation_InsertValues,
                )*);

                type PreOperation_Output = ($(
                    $T::PreOperation_Output,
                )*);

                type PreOperation_SqlStatement = ($(
                    $T::PreOperation_SqlStatement,
                )*);

                fn insert_identifiers(&self) -> Self::InsertIdentifiers {
                    ($(
                        Nest(paste::paste!(self.$idx.insert_identifier())),
                    )*)
                }

                type InsertIdentifiers = ($(
                    Nest<$T::InsertIdentifiers>,
                )*);

                fn insert_values(
                    &self,
                    init_insert_values: Self::Init_InsertValues,
                    pre_operation_insert_values: Self::PreOperation_InsertValues,
                ) -> Self::InsertValues {
                    ($(
                        Nest(paste::paste!(self.$idx.insert_values(
                            init_insert_values.$idx,
                            pre_operation_insert_values.$idx,
                        ))),
                    )*)
                }

                type InsertValues = ($(
                    Nest<$T::InsertValues>,
                )*);

                fn from_row(&self) -> Self::FromRow {
                    (
                        $(
                            paste::paste!(self.$idx.from_row()),
                        )*
                    )
                }

                type FromRow = ($(
                    $T::FromRow,
                )*);

                fn sql_statement_parts(
                    &self,
                    init_sql_statement: Self::Init_SqlStatement,
                    pre_operation_sql_statement: Self::PreOperation_SqlStatement,
                    from_row_data: <Self::FromRow as crate::from_row::FromRowData>::RData,
                ) -> super::SqlStatmentParts<Self::PostOperation, Self::SqlStatement_Output> {
                    let tuple = ($(
                        paste::paste!(self.$idx.sql_statement_parts(
                            pre_operation_sql_statement.$idx,
                            init_sql_statement.$idx,
                            from_row_data.$idx,
                        )),
                    )*);
                    super::SqlStatmentParts {
                        post_operation: ($(
                            paste::paste!(tuple.$idx.post_operation),
                        )*),
                        sql_statement_output: ($(
                            paste::paste!(tuple.$idx.sql_statement_output),
                        )*),
                    }
                }

                type PostOperation = ($(
                    $T::PostOperation,
                )*);

                type SqlStatement_Output = ($(
                    $T::SqlStatement_Output,
                )*);

                fn output(
                    self,
                    pre_operation_output: Self::PreOperation_Output,
                    sql_statement_output: Self::SqlStatement_Output,
                    post_operation_output: <Self::PostOperation as crate::operations::OperationOutput>::Output,
                ) -> Self::Output {
                    ($(
                        paste::paste!(self.$idx.output(
                            pre_operation_output.$idx,
                            sql_statement_output.$idx,
                            post_operation_output.$idx,
                        )),
                    )*)
                }

                type Output = ($(
                    $T::Output,
                )*);
            }
        };
    }

    impl_tuple!();
    impl_tuple!([0 T0] [1 T1]);
}

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;

    use crate::define_collection::collections_example::{
        Category, CategoryHandler, Todo, TodoHandler,
    };
    use crate::links::SetNew;
    use crate::operations_v2::insert_one::insert_id_link::InsertId;
    use crate::valid_syntax::MembersExpressions;
    use crate::{
        collections::CollectionId,
        define_collection::{Todo, TodoHandler},
        execute::Executable,
        extend_sqlx::InMemoryConnection,
        fix_executor::ExecutorTrait,
        operations_v2::fallibility::PanicOnFailure,
        sqlx_query_builder::{
            Expression,
            combinators::Nest,
            statements::insert_statement::{InsertOneValue, InsertStatement},
        },
        valid_syntax::{MembersAliasedExpressions, OnInsertHandler, TableExpressions},
    };

    use super::*;

    #[tokio::test]
    /// structure of the test
    /// - arguments typically passed to Operation::execute_operation
    /// - body of execute_operation coppied almost verbatim
    ///
    /// The purpose of this test is to escape the generic types errors
    /// and better visualize what is going on if all types are infered
    async fn insert_one_operation_body_types_infered() {
        let mut conn = Sqlite::in_memory_connection().await;
        let this = InsertOne {
            handler: TodoHandler,
            attributes: Todo {
                title: "test".to_string(),
                done: false,
                description: Some("test".to_string()),
            },
            link: (
                InsertId(4),
                SetNew {
                    link: CategoryHandler,
                    data: Category {
                        title: "test".to_string(),
                    },
                },
            ),
            infalibility: PanicOnFailure,
        };

        let link = this.link.init();
        let id = this.handler.id();

        let (sql, arg) = Expression::<Sqlite>::sql_statement(InsertStatement {
            table_name: TableExpressions::pascal_case(&this.handler),
            identifiers: (
                // attributes
                Nest(this.handler.identifiers()),
                // links
                Nest((
                    Nest(link.link_spec.0.insert_identifiers()),
                    Nest(link.link_spec.1.insert_identifiers()),
                )),
            ),
            values: InsertOneValue((
                // attributes
                Nest(this.handler.on_insert(this.attributes)),
                // links
                Nest((
                    Nest(link.link_spec.0.insert_identifiers()),
                    Nest(link.link_spec.1.insert_identifiers()),
                )),
            )),
            returning: (
                Nest(this.handler.identifiers()),
                Nest((
                    Nest(link.link_spec.0.insert_identifiers()),
                    Nest(link.link_spec.1.insert_identifiers()),
                )),
            ),
        });

        let res = <Sqlite as ExecutorTrait>::fetch_optional(
            &mut conn,
            Executable {
                string: &sql,
                arguments: arg,
            },
        ) // Pin<Box<Future<Output = ...>>>
        .await // Result<Option<SqliteRow>, sqlx::Error>
        .unwrap();
    }

    #[linked_ql_macros::skip]
    async fn test_insert_one() {
        let mut conn = Sqlite::in_memory_connection().await;
        let insert_one = Operation::<Sqlite>::exec_operation(
            InsertOneValue {
                handler: TodoHandler,
                data: InsertEntity {
                    attributes: Todo {
                        title: "test".to_string(),
                        done: false,
                        description: Some("test".to_string()),
                    },
                    link: (),
                },
                infalibility: PanicOnFailure,
            },
            &mut conn,
        )
        .await;
    }
}
