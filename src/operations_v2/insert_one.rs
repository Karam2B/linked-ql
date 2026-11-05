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
    operations_v2::FallibleOperationOutput, valid_syntax::OnInsert,
};

/// User facing types used in insert one operations.
/// Types implementing this trait are used directly by the user
/// to use a link in an insert one operation.
pub trait InsertOneLink<Base> {
    type LinkMeta: Send
        + InsertOneLinkMeta
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
    type Init_SqlStatement: Send;

    /// passed from `init` to `OnInsert::on_insert`
    type Init_OnInsert: Send;

    /// operation executed before the main sql statement, to implement Operation<S>
    type PreOperation: Send + FallibleOperationOutput;

    /// split PreOperation expensive data into parts
    fn pre_operation_parts(
        &self,
        pre_operation_output: <Self::PreOperation as FallibleOperationOutput>::Output,
    ) -> PreOperationParts<
        Self::PreOperation_OnInsert,
        Self::PreOperation_SqlStatement,
        Self::PreOperation_Output,
    >;

    /// passed from `pre_operation_parts` to `OnInsert::on_insert` method
    type PreOperation_OnInsert: Send;
    /// passed from `pre_operation_parts` to `output` method
    type PreOperation_Output: Send;
    /// passed from `pre_operation_parts` to `sql_statement` method
    type PreOperation_SqlStatement: Send;

    /// to retrieve members specified in INSERT INTO clause of the insert sql
    fn indentifiers(&self) -> Self::Identifiers;
    /// to retrieve members specified in INSERT INTO clause of the insert sql, to implement Join<Identifiers>: Expression<S, 'q>
    type Identifiers: Send;

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

    /// operation executed after the main sql statement, to implement FallibleOperation<S>
    type PostOperation: Send + FallibleOperationOutput;

    /// passed from `sql_statement` to `output`
    type SqlStatement_Output: Send;

    /// to return link's output
    fn output(
        self,
        pre_operation_output: Self::PreOperation_Output,
        sql_statement_output: Self::SqlStatement_Output,
        post_operation_output: <Self::PostOperation as FallibleOperationOutput>::Output,
    ) -> Self::Output;

    /// main data extracted by the link
    type Output: Send;
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
    use crate::extend_sqlx::database_error::{ErrorHandler, InfalibilityStrategy};
    use crate::extend_sqlx::DatabaseStatementBuilder;
    use crate::fix_executor::ExecutorTrait;
    use crate::from_row::FromRow;
    use crate::operations_v2::{FallibleOperation, FallibleOperationOutput};
    use crate::sqlx_query_builder::combinators::Nest;
    use crate::sqlx_query_builder::statements::insert_statement::{
        InsertOneValue, InsertStatement,
    };
    use crate::sqlx_query_builder::Expression;
    use crate::valid_syntax::{MembersExpressions, TableExpressions};
    use crate::{collections::CollectionMeta, operations::LinkedOutput};

    impl<Meta, Link, LinkSpec, Fallibility> FallibleOperationOutput
        for InsertOne<Meta, Link, Fallibility>
    where
        Fallibility: InfalibilityStrategy<Error: Send>,
        Link: InsertOneLink<Meta, LinkMeta = LinkSpec>,
        LinkSpec: InsertOneLinkMeta<Output: Send>,
        Meta: CollectionMeta<OutputData: Send, OutputIdData: Send>,
    {
        type Output = LinkedOutput<Meta::OutputIdData, Meta::OutputData, LinkSpec::Output>;
        type Error = Fallibility::Error;
    }

    impl<S, Meta, Link, LinkMeta, Fallibility> FallibleOperation<S>
        for InsertOne<Meta, Link, Fallibility>
    where
        Fallibility: Send + ErrorHandler<S, Error: Send>,
        S: DatabaseStatementBuilder + ExecutorTrait,
        Link: InsertOneLink<Meta, LinkMeta = LinkMeta> + Send,
        // prepare for sql statement
        LinkMeta: Send
            + InsertOneLinkMeta
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
        LinkMeta::PreOperation: FallibleOperation<S, Error: Into<Self::Error>>,
        LinkMeta::PostOperation: FallibleOperation<S, Error: Into<Self::Error>>,
        Meta: for<'r> FromRow<'r, S::Row, RData = Meta::OutputData>,
        Meta::IdMeta: Send + for<'r> FromRow<'r, S::Row, RData = Meta::OutputIdData>,
        for<'r> LinkMeta::FromRow: FromRow<'r, S::Row>,
    {
        fn exec_operation(
            self,
            conn: &mut S::Connection,
        ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send
        where
            S: sqlx::Database,
            Self: Sized,
        {
            async move {
                let id_meta = self.meta.id_meta();
                let InsertParts {
                    link_meta,
                    pre_operation,
                    init_on__insert,
                    init_sql__statement,
                } = self.link.init(self.meta.clone());

                let pre_operation_output = pre_operation
                    .exec_operation(conn)
                    .await
                    .map_err(Into::into)?;
                let pre_operation_parts = link_meta.pre_operation_parts(pre_operation_output);

                let (sql_string, arguments) = Expression::<'_, S>::sql_statement(InsertStatement {
                    table_name: self.meta.pascal_case(),
                    identifiers: (
                        Nest(self.meta.on_insert_identifiers()),
                        Nest(link_meta.on_insert_identifiers()),
                    ),
                    values: InsertOneValue((
                        Nest(self.meta.on_insert_values(self.attributes)),
                        Nest(link_meta.on_insert_values(OnInsertParts {
                            from_init: init_on__insert,
                            from_pre_operation: pre_operation_parts.pre_operation__on_insert,
                        })),
                    )),
                    returning: (
                        Nest(id_meta.identifiers()),
                        Nest(self.meta.identifiers()),
                        Nest(link_meta.indentifiers()),
                    ),
                });
                let row = <S as ExecutorTrait>::fetch_optional(
                    conn,
                    Executable {
                        string: sql_string.as_str(),
                        arguments,
                    },
                )
                .await
                .map_err(|err| self.infalibility.handle_sqlx_error(err))?
                .expect("bug: returning is specified");

                let id = id_meta
                    .from_row_no_alias(&row)
                    .expect("bug: id returning decode");
                let attributes = self
                    .meta
                    .from_row_no_alias(&row)
                    .expect("bug: attribute returning decode");
                let from_row_data = link_meta
                    .from_row()
                    .from_row_no_alias(&row)
                    .expect("bug: link returning decode");
                let sql_parts = link_meta.sql_statement_parts(
                    init_sql__statement,
                    pre_operation_parts.pre_operation__sql_statement,
                    from_row_data,
                );
                let post_operation_output = sql_parts
                    .post_operation
                    .exec_operation(conn)
                    .await
                    .map_err(Into::into)?;
                let links = link_meta.output(
                    pre_operation_parts.pre_operation__output,
                    sql_parts.sql_statement__output,
                    post_operation_output,
                );

                Ok(LinkedOutput {
                    id,
                    attributes,
                    links,
                })
            }
        }
    }
}

pub mod insert_id_link {
    use std::marker::PhantomData;

    use crate::{
        collections::CollectionMeta,
        operations_v2::insert_one::{InsertOneLink, InsertOneLinkMeta, OnInsertParts},
        operations_v2::FallibleOperationOutput,
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

    impl<IdMeta> InsertIdMeta<IdMeta> {
        pub fn new(id_meta: IdMeta) -> Self {
            InsertIdMeta(id_meta)
        }
    }

    impl<Base> InsertOneLink<Base> for InsertId<Base::InputIdData>
    where
        Base: CollectionMeta,
        Base::IdMeta: Send + for<'a> OnInsert<'a, InputData = Base::InputIdData>,
        Base::InputIdData: Send,
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

    impl<IdMeta: Send, OnInsertInputData: Send> InsertOneLinkMeta for InsertIdMeta<IdMeta>
    where
        IdMeta: for<'a> OnInsert<'a, InputData = OnInsertInputData>,
    {
        type Init_OnInsert = OnInsertInputData;

        type Init_SqlStatement = ();

        type PreOperation = ();

        fn pre_operation_parts(
            &self,
            _: <Self::PreOperation as FallibleOperationOutput>::Output,
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
            _: <Self::PostOperation as FallibleOperationOutput>::Output,
        ) -> Self::Output {
        }

        type Output = ();
    }
}

mod impl_insert_one_system_for_many_links {
    use crate::{
        operations_v2::{
            insert_one::{InsertOneLink, InsertOneLinkMeta, OnInsertParts},
            FallibleOperationOutput,
        },
        sqlx_query_builder::combinators::Nest,
        valid_syntax::OnInsert,
    };

    pub struct TupleMeta<T>(T);

    macro_rules! impl_insert_one_for_many_links {
        (()) => {
            impl_insert_one_for_many_links!(@impl);
        };
        (($($T:ident),+ $(,)?)) => {
            impl_insert_one_for_many_links!(@index [] [$($T),+] [0 1 2 3 4 5 6 7]);
        };
        (@index [$($done:tt)*] [] [$($_rest_idx:tt)*]) => {
            impl_insert_one_for_many_links!(@impl $($done)*);
        };
        (@index [$($done:tt)*] [$T:ident $(, $Trest:ident)*] [$idx:tt $($idx_rest:tt)*]) => {
            impl_insert_one_for_many_links!(@index [$($done)* $idx $T,] [$($Trest),*] [$($idx_rest)*]);
        };
        (@impl) => {
            impl<Base> InsertOneLink<Base> for ()
            where
                Base: Clone,
            {
                type LinkMeta = TupleMeta<()>;
                fn init(
                    self,
                    _base: Base,
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
                    _: <Self::PreOperation as FallibleOperationOutput>::Output,
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
                    _: <Self::PostOperation as FallibleOperationOutput>::Output,
                ) -> Self::Output {
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
        };
        (@impl $idx0:tt $T0:ident, $($idx:tt $T:ident,)*) => {
            impl<$T0, $($T,)* Base> InsertOneLink<Base> for ($T0, $($T,)*)
            where
                $T0: InsertOneLink<Base>,
                $($T: InsertOneLink<
                    Base,
                    LinkMeta: InsertOneLinkMeta<
                        PreOperation: FallibleOperationOutput<
                            Error = <<$T0::LinkMeta as InsertOneLinkMeta>::PreOperation as FallibleOperationOutput>::Error,
                        >,
                        PostOperation: FallibleOperationOutput<
                            Error = <<$T0::LinkMeta as InsertOneLinkMeta>::PostOperation as FallibleOperationOutput>::Error,
                        >,
                    >,
                >,)*
                Base: Clone,
            {
                type LinkMeta = TupleMeta<($T0::LinkMeta, $($T::LinkMeta,)*)>;
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
                        self.$idx0.init(base.clone()),
                        $(self.$idx.init(base.clone()),)*
                    );
                    super::InsertParts {
                        link_meta: TupleMeta((
                            init_tuple.$idx0.link_meta,
                            $(init_tuple.$idx.link_meta,)*
                        )),
                        pre_operation: (
                            init_tuple.$idx0.pre_operation,
                            $(init_tuple.$idx.pre_operation,)*
                        ),
                        init_on__insert: (
                            init_tuple.$idx0.init_on__insert,
                            $(init_tuple.$idx.init_on__insert,)*
                        ),
                        init_sql__statement: (
                            init_tuple.$idx0.init_sql__statement,
                            $(init_tuple.$idx.init_sql__statement,)*
                        ),
                    }
                }
            }

            impl<'a, $T0, $($T),*> OnInsert<'a> for TupleMeta<($T0, $($T,)*)>
            where
                $T0: InsertOneLinkMeta
                    + OnInsert<
                        'a,
                        InputData = OnInsertParts<$T0::Init_OnInsert, $T0::PreOperation_OnInsert>,
                    >,
                $($T: InsertOneLinkMeta
                    + OnInsert<
                        'a,
                        InputData = OnInsertParts<$T::Init_OnInsert, $T::PreOperation_OnInsert>,
                    >,)*
            {
                type InputData = OnInsertParts<
                    ($T0::Init_OnInsert, $($T::Init_OnInsert,)*),
                    ($T0::PreOperation_OnInsert, $($T::PreOperation_OnInsert,)*),
                >;
                type InsertValues = (Nest<$T0::InsertValues>, $(Nest<$T::InsertValues>,)*);
                type InsertIdentifiers = (Nest<$T0::InsertIdentifiers>, $(Nest<$T::InsertIdentifiers>,)*);

                fn on_insert_values(&'a self, input: Self::InputData) -> Self::InsertValues {
                    (
                        Nest(self.0.$idx0.on_insert_values(OnInsertParts {
                            from_init: input.from_init.$idx0,
                            from_pre_operation: input.from_pre_operation.$idx0,
                        })),
                        $(
                            Nest(self.0.$idx.on_insert_values(OnInsertParts {
                                from_init: input.from_init.$idx,
                                from_pre_operation: input.from_pre_operation.$idx,
                            })),
                        )*
                    )
                }

                fn on_insert_identifiers(&'a self) -> Self::InsertIdentifiers {
                    (
                        Nest(self.0.$idx0.on_insert_identifiers()),
                        $(Nest(self.0.$idx.on_insert_identifiers()),)*
                    )
                }
            }

            impl<$T0, $($T),*> InsertOneLinkMeta for TupleMeta<($T0, $($T,)*)>
            where
                $T0: InsertOneLinkMeta,
                $($T: InsertOneLinkMeta<
                    PreOperation: FallibleOperationOutput<
                        Error = <$T0::PreOperation as FallibleOperationOutput>::Error,
                    >,
                    PostOperation: FallibleOperationOutput<
                        Error = <$T0::PostOperation as FallibleOperationOutput>::Error,
                    >,
                >,)*
            {
                type Init_OnInsert = ($T0::Init_OnInsert, $($T::Init_OnInsert,)*);
                type Init_SqlStatement = ($T0::Init_SqlStatement, $($T::Init_SqlStatement,)*);
                type PreOperation = ($T0::PreOperation, $($T::PreOperation,)*);
                fn pre_operation_parts(
                    &self,
                    pre_operation_output: <Self::PreOperation as FallibleOperationOutput>::Output,
                ) -> super::PreOperationParts<
                    Self::PreOperation_OnInsert,
                    Self::PreOperation_SqlStatement,
                    Self::PreOperation_Output,
                > {
                    let tuple = (
                        self.0.$idx0.pre_operation_parts(pre_operation_output.$idx0),
                        $(self.0.$idx.pre_operation_parts(pre_operation_output.$idx),)*
                    );
                    super::PreOperationParts {
                        pre_operation__on_insert: (
                            tuple.$idx0.pre_operation__on_insert,
                            $(tuple.$idx.pre_operation__on_insert,)*
                        ),
                        pre_operation__output: (
                            tuple.$idx0.pre_operation__output,
                            $(tuple.$idx.pre_operation__output,)*
                        ),
                        pre_operation__sql_statement: (
                            tuple.$idx0.pre_operation__sql_statement,
                            $(tuple.$idx.pre_operation__sql_statement,)*
                        ),
                    }
                }
                type PreOperation_OnInsert = ($T0::PreOperation_OnInsert, $($T::PreOperation_OnInsert,)*);
                type PreOperation_Output = ($T0::PreOperation_Output, $($T::PreOperation_Output,)*);
                type PreOperation_SqlStatement = ($T0::PreOperation_SqlStatement, $($T::PreOperation_SqlStatement,)*);
                type Identifiers = (Nest<$T0::Identifiers>, $(Nest<$T::Identifiers>,)*);
                fn indentifiers(&self) -> Self::Identifiers {
                    (
                        Nest(self.0.$idx0.indentifiers()),
                        $(Nest(self.0.$idx.indentifiers()),)*
                    )
                }
                fn from_row(&self) -> Self::FromRow {
                    (
                        self.0.$idx0.from_row(),
                        $(self.0.$idx.from_row(),)*
                    )
                }
                type FromRow = ($T0::FromRow, $($T::FromRow,)*);
                fn sql_statement_parts(
                    &self,
                    init_sql_statement: Self::Init_SqlStatement,
                    pre_operation_sql_statement: Self::PreOperation_SqlStatement,
                    from_row_data: <Self::FromRow as crate::from_row::FromRowData>::RData,
                ) -> super::SqlStatmentParts<Self::PostOperation, Self::SqlStatement_Output> {
                    let tuple = (
                        self.0.$idx0.sql_statement_parts(
                            init_sql_statement.$idx0,
                            pre_operation_sql_statement.$idx0,
                            from_row_data.$idx0,
                        ),
                        $(
                            self.0.$idx.sql_statement_parts(
                                init_sql_statement.$idx,
                                pre_operation_sql_statement.$idx,
                                from_row_data.$idx,
                            ),
                        )*
                    );
                    super::SqlStatmentParts {
                        post_operation: (
                            tuple.$idx0.post_operation,
                            $(tuple.$idx.post_operation,)*
                        ),
                        sql_statement__output: (
                            tuple.$idx0.sql_statement__output,
                            $(tuple.$idx.sql_statement__output,)*
                        ),
                    }
                }
                type PostOperation = ($T0::PostOperation, $($T::PostOperation,)*);
                type SqlStatement_Output = ($T0::SqlStatement_Output, $($T::SqlStatement_Output,)*);
                fn output(
                    self,
                    pre_operation_output: Self::PreOperation_Output,
                    sql_statement_output: Self::SqlStatement_Output,
                    post_operation_output: <Self::PostOperation as FallibleOperationOutput>::Output,
                ) -> Self::Output {
                    (
                        self.0.$idx0.output(
                            pre_operation_output.$idx0,
                            sql_statement_output.$idx0,
                            post_operation_output.$idx0,
                        ),
                        $(
                            self.0.$idx.output(
                                pre_operation_output.$idx,
                                sql_statement_output.$idx,
                                post_operation_output.$idx,
                            ),
                        )*
                    )
                }
                type Output = ($T0::Output, $($T::Output,)*);
            }
        };
    }

    impl_insert_one_for_many_links!(());
    impl_insert_one_for_many_links!((T0,));
    impl_insert_one_for_many_links!((T0, T1));
}

#[cfg(test)]
mod tests {
    use sqlx::{Sqlite, SqliteConnection};

    use crate::collections::CollectionMeta;
    use crate::define_collection::collections_example::{Category, CategoryMeta, Todo, TodoMeta};
    use crate::extend_sqlx::database_error::{ErrorHandler, PropagateStateErrors, StateError};
    use crate::from_row::FromRow;
    use crate::links::SetNew;
    use crate::operations::LinkedOutput;
    use crate::operations_v2::insert_one::insert_id_link::InsertId;
    use crate::operations_v2::FallibleOperation;
    use crate::valid_syntax::MembersExpressions;
    use crate::{
        execute::Executable,
        extend_sqlx::InMemoryConnection,
        fix_executor::ExecutorTrait,
        sqlx_query_builder::{
            combinators::Nest,
            statements::insert_statement::{InsertOneValue, InsertStatement},
            Expression,
        },
        valid_syntax::{MembersAliasedExpressions, TableExpressions},
    };

    use super::*;

    type InsertOneOperation = InsertOne<TodoMeta, InsertId<i64>, PropagateStateErrors>;
    type InsertOneOperationResult = Result<LinkedOutput<i64, Todo, ()>, StateError>;

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
            infalibility: PropagateStateErrors,
        };

        let from_body = operation_body(this.clone(), &mut conn).await.unwrap();

        this.link = InsertId(5);

        let from_interface = operation_interface(this, &mut conn).await.unwrap();
        let attributes = Todo {
            title: "test".to_string(),
            done: false,
            description: Some("test".to_string()),
        };
        assert_eq!(
            from_body,
            LinkedOutput {
                id: 4,
                attributes: attributes.clone(),
                links: (),
            },
        );
        assert_eq!(
            from_interface,
            LinkedOutput {
                id: 5,
                attributes,
                links: (),
            },
        );
    }

    async fn operation_body(
        this: InsertOne<TodoMeta, InsertId<i64>, PropagateStateErrors>,
        conn: &mut SqliteConnection,
    ) -> InsertOneOperationResult {
        let id_meta = this.meta.id_meta();
        let InsertParts {
            link_meta,
            pre_operation,
            init_on__insert,
            init_sql__statement,
        } = this.link.init(this.meta.clone());

        let pre_operation_output =
            FallibleOperation::<Sqlite>::exec_operation(pre_operation, conn).await?;
        let pre_operation_parts = link_meta.pre_operation_parts(pre_operation_output);

        let (sql, arg) = Expression::<Sqlite>::sql_statement(InsertStatement {
            table_name: TableExpressions::pascal_case(&this.meta),
            identifiers: (
                // attributes
                Nest(this.meta.on_insert_identifiers()),
                // links
                Nest(link_meta.on_insert_identifiers()),
            ),
            values: InsertOneValue((
                // attributes
                Nest(this.meta.on_insert_values(this.attributes)),
                // links
                Nest(link_meta.on_insert_values(OnInsertParts {
                    from_init: init_on__insert,
                    from_pre_operation: pre_operation_parts.pre_operation__on_insert,
                })),
            )),
            returning: (
                Nest(id_meta.identifiers()),
                Nest(this.meta.identifiers()),
                Nest(link_meta.indentifiers()),
            ),
        });

        let row = <Sqlite as ExecutorTrait>::fetch_optional(
            conn,
            Executable {
                string: &sql,
                arguments: arg,
            },
        ) // Pin<Box<Future<Output = ...>>>
        .await // Result<Option<SqliteRow>, sqlx::Error>
        .map_err(|err| this.infalibility.handle_sqlx_error(err))?
        .unwrap();

        let id = id_meta
            .from_row_no_alias(&row)
            .expect("bug: id returning decode");
        let attributes = this
            .meta
            .from_row_no_alias(&row)
            .expect("bug: attribute returning decode");
        let from_row_data = link_meta
            .from_row()
            .from_row_no_alias(&row)
            .expect("bug: link returning decode");
        let sql_parts = link_meta.sql_statement_parts(
            init_sql__statement,
            pre_operation_parts.pre_operation__sql_statement,
            from_row_data,
        );
        let post_operation_output =
            FallibleOperation::<Sqlite>::exec_operation(sql_parts.post_operation, conn).await?;
        let links = link_meta.output(
            pre_operation_parts.pre_operation__output,
            sql_parts.sql_statement__output,
            post_operation_output,
        );

        Ok(LinkedOutput {
            id,
            attributes,
            links,
        })
    }

    async fn operation_interface(
        this: InsertOne<TodoMeta, InsertId<i64>, PropagateStateErrors>,
        conn: &mut SqliteConnection,
    ) -> InsertOneOperationResult {
        FallibleOperation::<Sqlite>::exec_operation(this, conn).await
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
