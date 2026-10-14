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

    pub trait InsertLinkParts {
        type LinkSpec: InsertOneLink;
        fn init_parts(
            self,
        ) -> InsertParts<
            Self::LinkSpec,
            <Self::LinkSpec as InsertOneLink>::PreOperation,
            <Self::LinkSpec as InsertOneLink>::Init_InsertValues,
            <Self::LinkSpec as InsertOneLink>::Init_AfterStatement,
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

    pub trait InsertOneLink {
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

    impl InsertLinkParts for () {
        type LinkSpec = ();

        fn init_parts(
            self,
        ) -> InsertParts<
            Self::LinkSpec,
            <Self::LinkSpec as InsertOneLink>::PreOperation,
            <Self::LinkSpec as InsertOneLink>::Init_InsertValues,
            <Self::LinkSpec as InsertOneLink>::Init_AfterStatement,
        > {
            InsertParts {
                link_spec: (),
                pre_operation: (),
                init_insert_values: (),
                init_after_statement: (),
            }
        }
    }

    impl InsertOneLink for () {
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

pub struct InsertOne<Handler, Data, Infalibility> {
    pub handler: Handler,
    pub data: Data,
    pub infalibility: Infalibility,
}

pub struct InsertEntity<Attributes, Link> {
    pub attributes: Attributes,
    pub link: Link,
}

mod impl_operation_for_insert_one {
    use super::*;
    use crate::operations::{Operation, OperationOutput};
    use crate::operations_v2::fallibility::PanicOnFailure;
    use crate::sqlx_query_builder::statements::insert_statement::InsertStatement;
    use crate::{
        collections::{Collection, CollectionId},
        operations::LinkedOutput,
    };

    impl<H, Link, LinkSpec> OperationOutput
        for InsertOne<H, InsertEntity<H::InputData, Link>, PanicOnFailure>
    where
        Link: InsertLinkParts<LinkSpec = LinkSpec>,
        LinkSpec: InsertOneLink,
        H: Collection,
    {
        type Output =
            LinkedOutput<<H::Id as CollectionId>::IdData, H::OutputData, LinkSpec::Output>;
    }

    impl<S, H, Link, LinkSpec> Operation<S>
        for InsertOne<H, InsertEntity<H::InputData, Link>, PanicOnFailure>
    where
        Link: InsertLinkParts<LinkSpec = LinkSpec>,
        LinkSpec: InsertOneLink,
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
        define_collection::{Todo, TodoHandler},
        execute::Executable,
        extend_sqlx::InMemoryConnection,
        fix_executor::ExecutorTrait,
        operations::Operation,
        operations_v2::fallibility::PanicOnFailure,
        sqlx_query_builder::{
            Expression,
            statements::insert_statement::{InsertOne, InsertStatement},
        },
        valid_syntax::{MembersExpressions, TableExpressions},
    };

    use super::*;

    #[tokio::test]
    async fn operation_body_types_infered() {
        let mut conn = Sqlite::in_memory_connection().await;
        let handler = TodoHandler;

        let (sql, arg) = Expression::<Sqlite>::sql_statement(InsertStatement {
            table_name: TableExpressions::pascal_case(&handler),
            identifiers: MembersExpressions::identifier(&handler),
            values: InsertOne(InsertEntity {
                attributes: Todo {
                    title: "test".to_string(),
                    done: false,
                    description: Some("test".to_string()),
                },
                link: (),
            }),
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
        )
        .await
        .unwrap();
    }

    #[linked_ql_macros::skip]
    async fn test_insert_one() {
        let mut conn = Sqlite::in_memory_connection().await;
        let insert_one = Operation::<Sqlite>::exec_operation(
            InsertOne {
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
