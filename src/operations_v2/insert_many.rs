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

#[allow(non_camel_case_types)]
pub mod insert_link_trait_system {
    use crate::{from_row::FromRowData, operations::OperationOutput};

    /// User facing types used in insert one operations.
    pub trait InsertOneLink {
        type LinkSpec: InsertOneLinkSpec;
        fn init(
            self,
        ) -> InsertParts<
            Self::LinkSpec,
            <Self::LinkSpec as InsertOneLinkSpec>::PreOperation,
            <Self::LinkSpec as InsertOneLinkSpec>::Init_InsertValues,
            <Self::LinkSpec as InsertOneLinkSpec>::Init_AfterStatement,
        >;
    }

    pub struct InsertParts<Spec, PreOperation, Init_InsertValues, Init_AfterStatement> {
        pub link_spec: Spec,
        pub pre_operation: PreOperation,
        pub init_insert_values: Init_InsertValues,
        pub init_after_statement: Init_AfterStatement,
    }

    pub struct PreOperationParts<
        PreOperation_InsertValues,
        PreOperation_Output,
        PreOperation_AfterStatement,
    > {
        pub pre_operation_insert_values: PreOperation_InsertValues,
        pub pre_operation_output: PreOperation_Output,
        pub pre_operation_after_statement: PreOperation_AfterStatement,
    }

    pub struct AfterStatmentParts<PostOperation, AfterStatment_Output> {
        pub post_operation: PostOperation,
        pub after_statement_output: AfterStatment_Output,
    }

    pub trait InsertOneLinkSpec {
        type Init_InsertValues;
        type Init_AfterStatement;
        type PreOperation: OperationOutput;

        fn pre_operation_parts(
            &self,
            pre_operation_output: <Self::PreOperation as OperationOutput>::Output,
        ) -> PreOperationParts<
            Self::PreOperation_InsertValues,
            Self::PreOperation_Output,
            Self::PreOperation_AfterStatement,
        >;

        type PreOperation_InsertValues;
        type PreOperation_Output;
        type PreOperation_AfterStatement;

        fn insert_returning(&self) -> Self::InsertReturning;
        type InsertReturning;

        fn insert_values(
            &self,
            init_insert_values: Self::Init_InsertValues,
            pre_operation_insert_values: Self::PreOperation_InsertValues,
        ) -> Self::InsertValues;
        type InsertValues;

        fn from_row(&self) -> Self::FromRow;
        type FromRow: FromRowData;

        fn after_statement_parts(
            &self,
            from_row_data: <Self::FromRow as FromRowData>::RData,
            init_after_statement: Self::Init_AfterStatement,
            pre_operation_after_statement: Self::PreOperation_AfterStatement,
        ) -> AfterStatmentParts<Self::PostOperation, Self::AfterStatement_Output>;
        type PostOperation: OperationOutput;
        type AfterStatement_Output;

        fn output(
            self,
            post_operation_output: <Self::PostOperation as OperationOutput>::Output,
            after_statement_output: Self::AfterStatement_Output,
            pre_operation_output: Self::PreOperation_Output,
        ) -> Self::Output;
        type Output;
    }
}

pub use insert_link_trait_system::*;

mod impl_insert_one_system_for_unit {
    use crate::{from_row::FromRowData, operations::OperationOutput};

    use super::*;

    impl InsertOneLink for () {
        type LinkSpec = ();

        fn init(
            self,
        ) -> InsertParts<
            Self::LinkSpec,
            <Self::LinkSpec as InsertOneLinkSpec>::PreOperation,
            <Self::LinkSpec as InsertOneLinkSpec>::Init_InsertValues,
            <Self::LinkSpec as InsertOneLinkSpec>::Init_AfterStatement,
        > {
            InsertParts {
                link_spec: (),
                pre_operation: (),
                init_insert_values: (),
                init_after_statement: (),
            }
        }
    }

    impl InsertOneLinkSpec for () {
        type Init_InsertValues = ();

        type Init_AfterStatement = ();

        type PreOperation = ();

        fn pre_operation_parts(
            &self,
            _: <Self::PreOperation as OperationOutput>::Output,
        ) -> PreOperationParts<
            Self::PreOperation_InsertValues,
            Self::PreOperation_Output,
            Self::PreOperation_AfterStatement,
        > {
            PreOperationParts {
                pre_operation_insert_values: (),
                pre_operation_output: (),
                pre_operation_after_statement: (),
            }
        }

        type PreOperation_InsertValues = ();

        type PreOperation_Output = ();

        type PreOperation_AfterStatement = ();

        fn insert_returning(&self) -> Self::InsertReturning {}

        type InsertReturning = ();

        fn insert_values(
            &self,
            _: Self::Init_InsertValues,
            _: Self::PreOperation_InsertValues,
        ) -> Self::InsertValues {
        }

        type InsertValues = ();

        fn from_row(&self) -> Self::FromRow {}

        type FromRow = ();

        fn after_statement_parts(
            &self,
            _: <Self::FromRow as FromRowData>::RData,
            _: Self::Init_AfterStatement,
            _: Self::PreOperation_AfterStatement,
        ) -> AfterStatmentParts<Self::PostOperation, Self::AfterStatement_Output> {
            AfterStatmentParts {
                post_operation: (),
                after_statement_output: (),
            }
        }

        type PostOperation = ();

        type AfterStatement_Output = ();

        fn output(
            self,
            _: <Self::PostOperation as OperationOutput>::Output,
            _: Self::AfterStatement_Output,
            _: Self::PreOperation_Output,
        ) -> Self::Output {
        }

        type Output = ();
    }
}

pub struct InsertMany<Handler, LinkHandler, Data, Infalibility> {
    pub handler: Handler,
    pub link_handler: LinkHandler,
    pub data: Data,
    pub infalibility: Infalibility,
}

pub struct InsertEntity<Attributes, Link> {
    pub attributes: Attributes,
    pub link: Link,
}

mod impl_operation_for_insert_many {
    use super::*;
    use crate::operations::{Operation, OperationOutput};
    use crate::operations_v2::fallibility::PanicOnFailure;
    use crate::sqlx_query_builder::statements::insert_statement::InsertStatement;
    use crate::{collections::Collection, operations::LinkedOutput};

    impl<Handler, Link, LinkSpec> OperationOutput
        for InsertOne<Handler, InsertEntity<Handler::InputData, Link>, PanicOnFailure>
    where
        Link: InsertOneLink<LinkSpec = LinkSpec>,
        LinkSpec: InsertOneLinkSpec,
        Handler: Collection,
    {
        type Output = LinkedOutput<Handler::OutputIdData, Handler::OutputData, LinkSpec::Output>;
    }

    impl<S, H, Link, LinkSpec> Operation<S>
        for InsertOne<H, InsertEntity<H::InputData, Link>, PanicOnFailure>
    where
        Link: InsertOneLink<LinkSpec = LinkSpec>,
        LinkSpec: InsertOneLinkSpec,
        H: Collection,
        Self: Send,
        Self::Output: Send,
    {
        fn exec_operation(self, _: &mut S::Connection) -> impl Future<Output = Self::Output> + Send
        where
            S: sqlx::Database,
            Self: Sized,
        {
            async move { todo!() }
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;

    use crate::{
        collections::{Collection, CollectionId},
        define_collection::collections_example::{Todo, TodoHandler},
        execute::Executable,
        extend_sqlx::InMemoryConnection,
        fix_executor::ExecutorTrait,
        operations::{IdOutput, Operation},
        operations_v2::fallibility::PanicOnFailure,
        sqlx_query_builder::{
            Expression,
            statements::insert_statement::{InsertOneValue, InsertStatement},
        },
        valid_syntax::{MembersExpressions, TableExpressions},
    };

    use super::*;

    #[linked_ql_macros::skip]
    #[tokio::test]
    async fn insert_many_operation_body_types_infered() {
        let mut conn = Sqlite::in_memory_connection().await;
        let handler = TodoHandler;

        let (sql, arg) = Expression::<Sqlite>::sql_statement(InsertStatement {
            table_name: TableExpressions::pascal_case(&handler),
            identifiers: (
                MembersExpressions::identifier(&handler),
                Nest((
                    ManualIdOfHandler(&handler),
                    MembersExpressions::identifier(&OneToMany {}),
                )),
            ),
            values: [
                InsertEntity {
                    attributes: Todo {
                        title: "test".to_string(),
                        done: false,
                        description: Some("test".to_string()),
                    },
                    link: (ManualId(3), IdOutput { id: 3 }),
                },
                InsertEntity {
                    attributes: Todo {
                        title: "test".to_string(),
                        done: false,
                        description: Some("test".to_string()),
                    },
                    link: (ManualId(4), IdOutput { id: 4 }),
                },
            ],
            returning: (
                MembersExpressions::identifier(&handler),
                MembersExpressions::identifier(&Collection::id(&handler)),
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

    #[tokio::test]
    /// structure of the test
    /// - arguments typically passed to Operation::execute_operation
    /// - body of execute_operation coppied almost verbatim
    ///
    /// The purpose of this test is to escape the generic types errors
    /// and better visualize what is going on if all types are infered
    async fn insert_one_operation_body_types_infered() {
        let mut conn = Sqlite::in_memory_connection().await;
        let this = InsertMany {
            handler: TodoHandler,
            link_handler: (
                InsertNew {
                    collection: CategoryHandler,
                    data: PhantomData::<Category>,
                },
                InsertId(PhantomData::<i64>),
            ),
            data: [
                InsertEntity {
                    attributes: Todo {},
                    link: (
                        Category {
                            title: "test".to_string,
                        },
                        3,
                    ),
                },
                InsertEntity {
                    attributes: Todo {},
                    link: (
                        Category {
                            title: "test".to_string,
                        },
                        4,
                    ),
                },
            ],
            infalibility: PanicOnFailure,
        };

        let (sql, arg) = Expression::<Sqlite>::sql_statement(InsertStatement {
            table_name: TableExpressions::pascal_case(&this.handler),
            identifiers: MembersExpressions::identifier(&this.handler),
            values: InsertOneValue(InsertEntity {
                attributes: this.data.attributes,
                link: this.data.link,
            }),
            returning: (
                MembersExpressions::identifier(&this.handler),
                MembersExpressions::identifier(&Collection::id(&this.handler)),
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
