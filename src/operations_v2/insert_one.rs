//! Operation for inserting one record
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
//! # Links
//!
//! Links are the standard way to extend operations.
//!
//! ## Trait system
//! To define a link working with OnInsert you have to implement three
//! traits and publish one type
//!
//! 1. define a user facing interface .. expected to be constructed by user
//!     and passed to InsertOne::link. Please provide documentation and
//!     public constructor. This type has to implement InsertOneLink trait.
//! 2. implement InsertOneLink: defines the starting point of the link, in
//!     addition to the "LinkMeta", which is private type, shared by
//!     reference and has to implement the rest of the system for the link to work
//! 3. implement OnInsert for LinkMeta: OnInsert define the valid syntax
//!     inside the sql statement
//! 4. implement OnInsertLinkMeta for LinkMeta: define the remaining
//!     behavior of the link.
//!
//! ## Multiple links
//! You can pass many links as tuple or vec of trait objects
//!

// ====================================
// === Insert One Link Trait System ===
// ====================================
use crate::{
    collections::CollectionMeta, from_row::FromRowData, operations::OperationOutput,
    valid_syntax::OnInsert,
};

/// User facing types used in insert one operations.
/// Types implementing this trait are used directly by the user
/// to use a link in an insert one operation.
pub trait InsertOneLink<Base> {
    type LinkMeta: InsertOneLinkMeta
        + for<'a> OnInsert<
            'a,
            InputData = OnInsertParts<
                <Self::LinkMeta as InsertOneLinkMeta>::Init_OnInsert,
                <Self::LinkMeta as InsertOneLinkMeta>::PreOperation_OnInsert,
            >,
        >;

    /// split expensive data to parts.
    fn init(
        self,
        base: Base,
    ) -> InsertParts<
        Self::LinkMeta,
        <Self::LinkMeta as InsertOneLinkMeta>::PreOperation,
        <Self::LinkMeta as InsertOneLinkMeta>::Init_OnInsert,
        <Self::LinkMeta as InsertOneLinkMeta>::Init_SqlStatement,
    >;
}

pub struct OnInsertParts<FromInit, FromPreOperation> {
    pub from_init: FromInit,
    pub from_pre_operation: FromPreOperation,
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct InsertParts<Meta, PreOperation, Init_OnInsert, Init_SqlStatement> {
    pub link_meta: Meta,
    pub pre_operation: PreOperation,
    pub init_on__insert: Init_OnInsert,
    pub init_sql__statement: Init_SqlStatement,
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct PreOperationParts<PreOperation_OnInsert, PreOperation_SqlStatement, PreOperation_Output>
{
    pub pre_operation__on_insert: PreOperation_OnInsert,
    pub pre_operation__sql_statement: PreOperation_SqlStatement,
    pub pre_operation__output: PreOperation_Output,
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct SqlStatmentParts<PostOperation, SqlStatment_Output> {
    pub post_operation: PostOperation,
    pub sql_statement__output: SqlStatment_Output,
}

/// link implementation used internally in InsertOne
///
/// Data extracted by `Operation` or `FromRowAlias` is assumed
/// to be expensive and cloning is discouraged.
#[allow(non_camel_case_types)]
pub trait InsertOneLinkMeta {
    /// passed from `init` to `sql_statement` method
    type Init_SqlStatement;

    /// passed from `init` to `OnInsert::on_insert`
    type Init_OnInsert;

    /// operation executed before the main sql statement, to implement Operation<S>
    type PreOperation: OperationOutput;

    /// split PreOperation expensive data into parts
    fn pre_operation_parts(
        &self,
        pre_operation_output: <Self::PreOperation as OperationOutput>::Output,
    ) -> PreOperationParts<
        Self::PreOperation_OnInsert,
        Self::PreOperation_SqlStatement,
        Self::PreOperation_Output,
    >;

    /// passed from `pre_operation_parts` to `OnInsert::on_insert` method
    type PreOperation_OnInsert;
    /// passed from `pre_operation_parts` to `output` method
    type PreOperation_Output;
    /// passed from `pre_operation_parts` to `sql_statement` method
    type PreOperation_SqlStatement;

    /// to retrieve members specified in INSERT INTO clause of the insert sql
    fn indentifiers(&self) -> Self::Identifiers;
    /// to retrieve members specified in INSERT INTO clause of the insert sql, to implement Join<Identifiers>: Expression<S, 'q>
    type Identifiers;

    /// to retrieve members specified in RETURNING clause of the insert sql
    fn from_row(&self) -> Self::FromRow;
    /// to retrieve members specified in RETURNING clause of the insert sql, to implement FromRowAlias<'r, S::Row>.
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
pub struct InsertOne<Meta: CollectionMeta, Link, Infalibility> {
    pub meta: Meta,
    pub attributes: <Meta as CollectionMeta>::InputData,
    pub link: Link,
    pub infalibility: Infalibility,
}

mod impl_operation_for_insert_one {
    use super::*;
    use crate::execute::Executable;
    use crate::extend_sqlx::DatabaseStatementBuilder;
    use crate::fix_executor::ExecutorTrait;
    use crate::operations::{Operation, OperationOutput};
    use crate::operations_v2::fallibility::PanicOnFailure;
    use crate::sqlx_query_builder::Expression;
    use crate::sqlx_query_builder::combinators::Nest;
    use crate::sqlx_query_builder::statements::insert_statement::{
        InsertOneValue, InsertStatement,
    };
    use crate::valid_syntax::{MembersExpressions, TableExpressions};
    use crate::{collections::CollectionMeta, operations::LinkedOutput};

    impl<Meta, Link, LinkSpec> OperationOutput for InsertOne<Meta, Link, PanicOnFailure>
    where
        Link: InsertOneLink<Meta, LinkMeta = LinkSpec>,
        LinkSpec: InsertOneLinkMeta,
        Meta: CollectionMeta,
    {
        type Output = LinkedOutput<Meta::OutputIdData, Meta::OutputData, LinkSpec::Output>;
    }

    impl<S, Meta, Link, LinkMeta> Operation<S> for InsertOne<Meta, Link, PanicOnFailure>
    where
        S: DatabaseStatementBuilder + ExecutorTrait,
        Link: InsertOneLink<Meta, LinkMeta = LinkMeta> + Send,
        // prepare for sql statement
        LinkMeta: InsertOneLinkMeta
            + for<'a> OnInsert<
                'a,
                InputData = OnInsertParts<
                    <LinkMeta as InsertOneLinkMeta>::Init_OnInsert,
                    <LinkMeta as InsertOneLinkMeta>::PreOperation_OnInsert,
                >,
            >,
        LinkMeta::Output: Send,
        Meta: CollectionMeta<IdMeta: for<'a> MembersExpressions<'a>>
            + for<'a> TableExpressions<'a>
            + for<'a> MembersExpressions<'a>
            + for<'a> OnInsert<'a, InputData = <Meta as CollectionMeta>::InputData>
            + Clone // Clone: a version of the meta is sent to the link
            + Send,
        Meta::OutputData: Send,
        Meta::OutputIdData: Send,
        <Meta as CollectionMeta>::InputData: Send,
        for<'a, 'q> InsertStatement<
            <Meta as TableExpressions<'a>>::PascalCase,
            (
                Nest<<Meta as OnInsert<'a>>::InsertIdentifiers>,
                Nest<<LinkMeta as OnInsert<'a>>::InsertIdentifiers>,
            ),
            InsertOneValue<(
                Nest<<Meta as OnInsert<'a>>::InsertValues>,
                Nest<<LinkMeta as OnInsert<'a>>::InsertValues>,
            )>,
            (
                Nest<<Meta::IdMeta as MembersExpressions<'a>>::Identifiers>,
                Nest<<Meta as MembersExpressions<'a>>::Identifiers>,
                Nest<<LinkMeta as InsertOneLinkMeta>::Identifiers>,
            ),
        >: Expression<'q, S>,
    {
        fn exec_operation(
            self,
            conn: &mut S::Connection,
        ) -> impl Future<Output = Self::Output> + Send
        where
            S: sqlx::Database,
            Self: Sized,
        {
            async move {
                let link = self.link.init(self.meta.clone());

                let (sql_string, arguments) = Expression::<'_, S>::sql_statement(InsertStatement {
                    table_name: self.meta.pascal_case(),
                    identifiers: (
                        Nest(self.meta.on_insert_identifiers()),
                        Nest(link.link_meta.on_insert_identifiers()),
                    ),
                    values: InsertOneValue((
                        Nest(self.meta.on_insert_values(self.attributes)),
                        Nest(link.link_meta.on_insert_values(OnInsertParts {
                            from_init: todo!(),
                            from_pre_operation: todo!(),
                        })),
                    )),
                    returning: (
                        Nest(self.meta.id_meta().identifiers()),
                        Nest(self.meta.identifiers()),
                        Nest(link.link_meta.indentifiers()),
                    ),
                });
                let row = <S as ExecutorTrait>::fetch_optional(
                    conn,
                    Executable {
                        string: sql_string.as_str(),
                        arguments: arguments,
                    },
                )
                .await
                .expect("bug is assumed .. but it might be constraint violation")
                .expect("bug: returning is specified");
            }
        }
    }
}

pub mod insert_id_link {
    use std::marker::PhantomData;

    use crate::{
        collections::CollectionMeta,
        operations_v2::insert_one::{InsertOneLink, InsertOneLinkMeta, OnInsertParts},
        valid_syntax::OnInsert,
    };

    /// To insert an id in InsertOne and InsertMany operations.
    ///
    /// example:
    /// ```rust
    /// let result = Operation::execute(InsertOne {
    ///     meta: TodoMeta,
    ///     attributes: Todo {
    ///         .. some code ..
    ///     },
    ///     link: InsertId(3),
    ///     infalibility: PanicOnFailure,
    /// }, &mut connection).await;
    ///
    /// assert_eq!(result.link, 3);
    ///
    /// ```
    #[derive(Debug, Clone)]
    pub struct InsertId<InputIdData>(pub InputIdData);

    pub struct InsertIdMeta<IdMeta>(IdMeta);

    impl<Base> InsertOneLink<Base> for InsertId<Base::InputIdData>
    where
        Base: CollectionMeta,
        Base::IdMeta: for<'a> OnInsert<'a, InputData = Base::InputIdData>,
    {
        type LinkMeta = InsertIdMeta<Base::IdMeta>;

        fn init(
            self,
            base: Base,
        ) -> super::InsertParts<
            Self::LinkMeta,
            <Self::LinkMeta as super::InsertOneLinkMeta>::PreOperation,
            <Self::LinkMeta as super::InsertOneLinkMeta>::Init_OnInsert,
            <Self::LinkMeta as super::InsertOneLinkMeta>::Init_SqlStatement,
        > {
            super::InsertParts {
                link_meta: InsertIdMeta(base.id_meta()),
                pre_operation: (),
                init_on__insert: self.0,
                init_sql__statement: (),
            }
        }
    }

    impl<'a, IdMeta> OnInsert<'a> for InsertIdMeta<IdMeta>
    where
        IdMeta: OnInsert<'a>,
    {
        type InputData = OnInsertParts<IdMeta::InputData, ()>;

        type InsertValues = IdMeta::InsertValues;

        type InsertIdentifiers = IdMeta::InsertIdentifiers;

        fn on_insert_values(&'a self, input: Self::InputData) -> Self::InsertValues {
            self.0.on_insert_values(input.from_init)
        }

        fn on_insert_identifiers(&'a self) -> Self::InsertIdentifiers {
            self.0.on_insert_identifiers()
        }
    }

    impl<IdMeta, OnInsertInputData> InsertOneLinkMeta for InsertIdMeta<IdMeta>
    where
        IdMeta: for<'a> OnInsert<'a, InputData = OnInsertInputData>,
    {
        type Init_OnInsert = OnInsertInputData;

        type Init_SqlStatement = ();

        type PreOperation = ();

        fn pre_operation_parts(
            &self,
            _: <Self::PreOperation as crate::operations::OperationOutput>::Output,
        ) -> super::PreOperationParts<
            Self::PreOperation_OnInsert,
            Self::PreOperation_SqlStatement,
            Self::PreOperation_Output,
        > {
            super::PreOperationParts {
                pre_operation__on_insert: (),
                pre_operation__output: (),
                pre_operation__sql_statement: (),
            }
        }

        type Identifiers = ();
        fn indentifiers(&self) -> Self::Identifiers {}

        type PreOperation_OnInsert = ();

        type PreOperation_Output = ();

        type PreOperation_SqlStatement = ();

        fn from_row(&self) -> Self::FromRow {}

        type FromRow = ();

        fn sql_statement_parts(
            &self,
            _: Self::Init_SqlStatement,
            _: Self::PreOperation_SqlStatement,
            _: <Self::FromRow as crate::from_row::FromRowData>::RData,
        ) -> super::SqlStatmentParts<Self::PostOperation, Self::SqlStatement_Output> {
            super::SqlStatmentParts {
                post_operation: (),
                sql_statement__output: (),
            }
        }

        type PostOperation = ();

        type SqlStatement_Output = ();

        fn output(
            self,
            _: Self::PreOperation_Output,
            _: Self::SqlStatement_Output,
            _: <Self::PostOperation as crate::operations::OperationOutput>::Output,
        ) -> Self::Output {
        }

        type Output = ();
    }
}

mod impl_insert_one_system_for_many_links {
    use crate::{
        operations_v2::insert_one::{InsertOneLink, InsertOneLinkMeta, OnInsertParts},
        sqlx_query_builder::combinators::Nest,
        valid_syntax::OnInsert,
    };

    pub struct TupleMeta<T>(T);

    macro_rules! impl_tuple {
        ($([$idx:literal $element:ident])*) => {

            #[allow(unused)]
            impl<$($element,)* Base> InsertOneLink<Base> for ($($element,)*)
            where $(
                $element: InsertOneLink<Base>,
            )* Base: Clone,
            {
                type LinkSpec = ($(
                    $element::LinkSpec,
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
                        paste::paste!(self.$idx.init(base.clone())),
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
            impl<$($element,)*> InsertOneLinkSpec for ($($element,)*)
            where $(
                $element: InsertOneLinkSpec,
            )*
            {
                type Init_InsertValues = ($(
                    $element::Init_InsertValues,
                )*);

                type Init_SqlStatement = ($(
                    $element::Init_SqlStatement,
                )*);

                type PreOperation = ($(
                    $element::PreOperation,
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
                    $element::PreOperation_InsertValues,
                )*);

                type PreOperation_Output = ($(
                    $element::PreOperation_Output,
                )*);

                type PreOperation_SqlStatement = ($(
                    $element::PreOperation_SqlStatement,
                )*);

                fn insert_identifiers(&self) -> Self::InsertIdentifiers {
                    ($(
                        Nest(paste::paste!(self.$idx.insert_identifiers())),
                    )*)
                }

                type InsertIdentifiers = ($(
                    Nest<$element::InsertIdentifiers>,
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
                    Nest<$element::InsertValues>,
                )*);

                fn from_row(&self) -> Self::FromRow {
                    (
                        $(
                            paste::paste!(self.$idx.from_row()),
                        )*
                    )
                }

                type FromRow = ($(
                    $element::FromRow,
                )*);

                fn sql_statement_parts(
                    &self,
                    init_sql_statement: Self::Init_SqlStatement,
                    pre_operation_sql_statement: Self::PreOperation_SqlStatement,
                    from_row_data: <Self::FromRow as crate::from_row::FromRowData>::RData,
                ) -> super::SqlStatmentParts<Self::PostOperation, Self::SqlStatement_Output> {
                    let tuple = ($(
                        paste::paste!(self.$idx.sql_statement_parts(
                            init_sql_statement.$idx,
                            pre_operation_sql_statement.$idx,
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
                    $element::PostOperation,
                )*);

                type SqlStatement_Output = ($(
                    $element::SqlStatement_Output,
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
                    $element::Output,
                )*);
            }
        };
    }

    impl<Base> InsertOneLink<Base> for ()
    where
        Base: Clone,
    {
        type LinkMeta = TupleMeta<()>;
        fn init(
            self,
            base: Base,
        ) -> super::InsertParts<
            Self::LinkMeta,
            <Self::LinkMeta as super::InsertOneLinkMeta>::PreOperation,
            <Self::LinkMeta as super::InsertOneLinkMeta>::Init_OnInsert,
            <Self::LinkMeta as super::InsertOneLinkMeta>::Init_SqlStatement,
        > {
            super::InsertParts {
                link_meta: TupleMeta(()),
                pre_operation: (),
                init_on__insert: (),
                init_sql__statement: (),
            }
        }
    }

    impl InsertOneLinkMeta for TupleMeta<()> {
        type Init_OnInsert = ();
        type Init_SqlStatement = ();
        type PreOperation = ();
        type Identifiers = ();
        fn indentifiers(&self) -> Self::Identifiers {}
        fn pre_operation_parts(
            &self,
            _: <Self::PreOperation as crate::operations::OperationOutput>::Output,
        ) -> super::PreOperationParts<
            Self::PreOperation_OnInsert,
            Self::PreOperation_SqlStatement,
            Self::PreOperation_Output,
        > {
            let tuple = ();
            super::PreOperationParts {
                pre_operation__on_insert: (),
                pre_operation__output: (),
                pre_operation__sql_statement: (),
            }
        }
        type PreOperation_OnInsert = ();
        type PreOperation_Output = ();
        type PreOperation_SqlStatement = ();
        fn from_row(&self) -> Self::FromRow {
            ()
        }
        type FromRow = ();
        fn sql_statement_parts(
            &self,
            _: Self::Init_SqlStatement,
            _: Self::PreOperation_SqlStatement,
            _: <Self::FromRow as crate::from_row::FromRowData>::RData,
        ) -> super::SqlStatmentParts<Self::PostOperation, Self::SqlStatement_Output> {
            super::SqlStatmentParts {
                post_operation: (),
                sql_statement__output: (),
            }
        }
        type PostOperation = ();
        type SqlStatement_Output = ();
        fn output(
            self,
            _: Self::PreOperation_Output,
            _: Self::SqlStatement_Output,
            _: <Self::PostOperation as crate::operations::OperationOutput>::Output,
        ) -> Self::Output {
            ()
        }
        type Output = ();
    }

    impl<'a> OnInsert<'a> for TupleMeta<()> {
        type InputData = OnInsertParts<(), ()>;

        type InsertValues = ();

        type InsertIdentifiers = ();

        fn on_insert_values(&'a self, _: Self::InputData) -> Self::InsertValues {}

        fn on_insert_identifiers(&'a self) -> Self::InsertIdentifiers {}
    }

    const _: () = {
        #[allow(unused)]
        impl<T0, T1, Base> InsertOneLink<Base> for (T0, T1)
        where
            T0: InsertOneLink<Base>,
            T1: InsertOneLink<Base>,
            Base: Clone,
        {
            type LinkMeta = TupleMeta<(T0::LinkMeta, T1::LinkMeta)>;
            fn init(
                self,
                base: Base,
            ) -> super::InsertParts<
                Self::LinkMeta,
                <Self::LinkMeta as super::InsertOneLinkMeta>::PreOperation,
                <Self::LinkMeta as super::InsertOneLinkMeta>::Init_OnInsert,
                <Self::LinkMeta as super::InsertOneLinkMeta>::Init_SqlStatement,
            > {
                let init_tuple = (
                    paste::paste!(self.0.init(base.clone())),
                    paste::paste!(self.1.init(base.clone())),
                );
                super::InsertParts {
                    link_meta: TupleMeta((
                        paste::paste!(init_tuple.0.link_meta),
                        paste::paste!(init_tuple.1.link_meta),
                    )),
                    pre_operation: (
                        paste::paste!(init_tuple.0.pre_operation),
                        paste::paste!(init_tuple.1.pre_operation),
                    ),
                    init_on__insert: (
                        paste::paste!(init_tuple.0.init_on__insert),
                        paste::paste!(init_tuple.1.init_on__insert),
                    ),
                    init_sql__statement: (
                        paste::paste!(init_tuple.0.init_sql__statement),
                        paste::paste!(init_tuple.1.init_sql__statement),
                    ),
                }
            }
        }

        impl<'a, T0, T1> OnInsert<'a> for TupleMeta<(T0, T1)>
        where
            T0: InsertOneLinkMeta
                + OnInsert<
                    'a,
                    InputData = OnInsertParts<T0::Init_OnInsert, T0::PreOperation_OnInsert>,
                >,
            T1: InsertOneLinkMeta
                + OnInsert<
                    'a,
                    InputData = OnInsertParts<T1::Init_OnInsert, T1::PreOperation_OnInsert>,
                >,
        {
            type InputData = OnInsertParts<
                (T0::Init_OnInsert, T1::Init_OnInsert),
                (T0::PreOperation_OnInsert, T1::PreOperation_OnInsert),
            >;

            type InsertValues = (Nest<T0::InsertValues>, Nest<T1::InsertValues>);

            type InsertIdentifiers = (Nest<T0::InsertIdentifiers>, Nest<T1::InsertIdentifiers>);

            fn on_insert_values(&'a self, input: Self::InputData) -> Self::InsertValues {
                (
                    Nest(self.0.0.on_insert_values(OnInsertParts {
                        from_init: input.from_init.0,
                        from_pre_operation: input.from_pre_operation.0,
                    })),
                    Nest(self.0.1.on_insert_values(OnInsertParts {
                        from_init: input.from_init.1,
                        from_pre_operation: input.from_pre_operation.1,
                    })),
                )
            }

            fn on_insert_identifiers(&'a self) -> Self::InsertIdentifiers {
                (
                    Nest(self.0.0.on_insert_identifiers()),
                    Nest(self.0.1.on_insert_identifiers()),
                )
            }
        }

        #[allow(unused)]
        impl<T0, T1> InsertOneLinkMeta for TupleMeta<(T0, T1)>
        where
            T0: InsertOneLinkMeta,
            T1: InsertOneLinkMeta,
        {
            type Init_OnInsert = (T0::Init_OnInsert, T1::Init_OnInsert);
            type Init_SqlStatement = (T0::Init_SqlStatement, T1::Init_SqlStatement);
            type PreOperation = (T0::PreOperation, T1::PreOperation);
            fn pre_operation_parts(
                &self,
                pre_operation_output: <Self::PreOperation as crate::operations::OperationOutput>::Output,
            ) -> super::PreOperationParts<
                Self::PreOperation_OnInsert,
                Self::PreOperation_SqlStatement,
                Self::PreOperation_Output,
            > {
                let tuple = (
                    paste::paste!(self.0.0.pre_operation_parts(pre_operation_output.0,)),
                    paste::paste!(self.0.1.pre_operation_parts(pre_operation_output.1,)),
                );
                super::PreOperationParts {
                    pre_operation__on_insert: (
                        paste::paste!(tuple.0.pre_operation__on_insert),
                        paste::paste!(tuple.1.pre_operation__on_insert),
                    ),
                    pre_operation__output: (
                        paste::paste!(tuple.0.pre_operation__output),
                        paste::paste!(tuple.1.pre_operation__output),
                    ),
                    pre_operation__sql_statement: (
                        paste::paste!(tuple.0.pre_operation__sql_statement),
                        paste::paste!(tuple.1.pre_operation__sql_statement),
                    ),
                }
            }
            type PreOperation_OnInsert = (T0::PreOperation_OnInsert, T1::PreOperation_OnInsert);
            type PreOperation_Output = (T0::PreOperation_Output, T1::PreOperation_Output);
            type PreOperation_SqlStatement =
                (T0::PreOperation_SqlStatement, T1::PreOperation_SqlStatement);
            fn from_row(&self) -> Self::FromRow {
                (
                    paste::paste!(self.0.0.from_row()),
                    paste::paste!(self.0.1.from_row()),
                )
            }
            type Identifiers = (Nest<T0::Identifiers>, Nest<T1::Identifiers>);
            fn indentifiers(&self) -> Self::Identifiers {
                (Nest(self.0.0.indentifiers()), Nest(self.0.1.indentifiers()))
            }
            type FromRow = (T0::FromRow, T1::FromRow);
            fn sql_statement_parts(
                &self,
                init_sql_statement: Self::Init_SqlStatement,
                pre_operation_sql_statement: Self::PreOperation_SqlStatement,
                from_row_data: <Self::FromRow as crate::from_row::FromRowData>::RData,
            ) -> super::SqlStatmentParts<Self::PostOperation, Self::SqlStatement_Output>
            {
                let tuple = (
                    paste::paste!(self.0.0.sql_statement_parts(
                        init_sql_statement.0,
                        pre_operation_sql_statement.0,
                        from_row_data.0,
                    )),
                    paste::paste!(self.0.1.sql_statement_parts(
                        init_sql_statement.1,
                        pre_operation_sql_statement.1,
                        from_row_data.1,
                    )),
                );
                super::SqlStatmentParts {
                    post_operation: (
                        paste::paste!(tuple.0.post_operation),
                        paste::paste!(tuple.1.post_operation),
                    ),
                    sql_statement__output: (
                        paste::paste!(tuple.0.sql_statement__output),
                        paste::paste!(tuple.1.sql_statement__output),
                    ),
                }
            }
            type PostOperation = (T0::PostOperation, T1::PostOperation);
            type SqlStatement_Output = (T0::SqlStatement_Output, T1::SqlStatement_Output);
            fn output(
                self,
                pre_operation_output: Self::PreOperation_Output,
                sql_statement_output: Self::SqlStatement_Output,
                post_operation_output: <Self::PostOperation as crate::operations::OperationOutput>::Output,
            ) -> Self::Output {
                (
                    paste::paste!(self.0.0.output(
                        pre_operation_output.0,
                        sql_statement_output.0,
                        post_operation_output.0,
                    )),
                    paste::paste!(self.0.1.output(
                        pre_operation_output.1,
                        sql_statement_output.1,
                        post_operation_output.1,
                    )),
                )
            }
            type Output = (T0::Output, T1::Output);
        }
    };
}

#[cfg(test)]
mod tests {
    use sqlx::{Sqlite, SqliteConnection};

    use crate::collections::CollectionMeta;
    use crate::define_collection::collections_example::{Category, CategoryMeta, Todo, TodoMeta};
    use crate::links::SetNew;
    use crate::operations::Operation;
    use crate::operations_v2::insert_one::insert_id_link::InsertId;
    use crate::valid_syntax::MembersExpressions;
    use crate::{
        execute::Executable,
        extend_sqlx::InMemoryConnection,
        fix_executor::ExecutorTrait,
        operations_v2::fallibility::PanicOnFailure,
        sqlx_query_builder::{
            Expression,
            combinators::Nest,
            statements::insert_statement::{InsertOneValue, InsertStatement},
        },
        valid_syntax::{MembersAliasedExpressions, TableExpressions},
    };

    use super::*;

    type InsertOneOperation = InsertOne<TodoMeta, InsertId<i64>, PanicOnFailure>;
    type InsertOneOperationResult = ();

    #[tokio::test]
    /// structure of the test
    /// - migrate, create input
    /// - pass input to `operation_body` -- is where the body of `execute_operation`` coppied almost verbatim
    /// - pass input to `operation_interface` -- is where `execute_operation` is called
    ///
    /// The purpose of this test is to better visualize what is going on if all types are infered
    /// which will help in debugging and understanding the code
    async fn insert_one() {
        let mut conn = Sqlite::in_memory_connection().await;

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS todo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                description TEXT
            );
        ",
        )
        .execute(&mut conn)
        .await
        .unwrap();

        let mut this = InsertOne {
            meta: TodoMeta,
            attributes: Todo {
                title: "test".to_string(),
                done: false,
                description: Some("test".to_string()),
            },
            link: InsertId(4),
            infalibility: PanicOnFailure,
        };

        // operation_body(this.clone(), &mut conn).await;

        // this.link = InsertId(5);

        operation_interface(this, &mut conn).await;
    }

    async fn operation_body(
        this: InsertOne<TodoMeta, InsertId<i64>, PanicOnFailure>,
        conn: &mut SqliteConnection,
    ) -> InsertOneOperationResult {
        let id_meta = this.meta.id_meta();
        let link = this.link.init(this.meta.clone());

        let (sql, arg) = Expression::<Sqlite>::sql_statement(InsertStatement {
            table_name: TableExpressions::pascal_case(&this.meta),
            identifiers: (
                // attributes
                Nest(this.meta.on_insert_identifiers()),
                // links
                Nest(link.link_meta.on_insert_identifiers()),
            ),
            values: InsertOneValue((
                // attributes
                Nest(this.meta.on_insert_values(this.attributes)),
                // links
                Nest(link.link_meta.on_insert_values(OnInsertParts {
                    from_init: link.init_on__insert,
                    from_pre_operation: (),
                })),
            )),
            returning: (
                Nest(id_meta.identifiers()),
                Nest(this.meta.identifiers()),
                Nest(link.link_meta.on_insert_identifiers()),
            ),
        });

        let res = <Sqlite as ExecutorTrait>::fetch_optional(
            conn,
            Executable {
                string: &sql,
                arguments: arg,
            },
        ) // Pin<Box<Future<Output = ...>>>
        .await // Result<Option<SqliteRow>, sqlx::Error>
        .unwrap();
    }

    async fn operation_interface(
        this: InsertOne<TodoMeta, InsertId<i64>, PanicOnFailure>,
        conn: &mut SqliteConnection,
    ) -> InsertOneOperationResult {
        Operation::<Sqlite>::exec_operation(this, conn).await;
    }
}

mod std_trait_impls {
    use super::*;

    impl<Meta, Link, Infalibility> Clone for InsertOne<Meta, Link, Infalibility>
    where
        Meta: CollectionMeta + Clone,
        Meta::InputData: Clone,
        Link: Clone,
        Infalibility: Clone,
    {
        fn clone(&self) -> Self {
            Self {
                meta: self.meta.clone(),
                attributes: self.attributes.clone(),
                link: self.link.clone(),
                infalibility: self.infalibility.clone(),
            }
        }
    }
}
