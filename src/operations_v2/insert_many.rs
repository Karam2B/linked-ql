//! Operation for inserting many records in one statement.
//!

//!
//! IteratorMeta: the container type of the input data will be the same as the container type of the output data
//! for example: `InsertMany<..., [InsertEntity<Todo, ()>; 2],...>` will have output of type `[LinkedOutput<...>; 2]`

/// InsertMany takes 4 generic parameters:
/// - Meta: the collection meta
/// - PsuedoLink: the psuedo link, usually has the same shape as LinkData (look at Data) but with data replaced by a PhantomData
/// - Data: iterator of InsertEntity<Meta::InputData, LinkData>,
///     the container type here will be the same as the container type of the output data
///     look [here](crate::operations_v2::insert_many::iterator_meta::IteratorMeta) for more details
/// - Infalibility: the infalibility strategy -- usually PropagateStateErrors
pub struct InsertMany<Meta, PsuedoLink, Data, Infalibility> {
    pub meta: Meta,
    pub psuedo_link: PsuedoLink,
    pub data: Data,
    pub infalibility: Infalibility,
}

pub struct InsertEntity<Attributes, Link> {
    pub attributes: Attributes,
    pub link: Link,
}

pub trait InsertManyLink<Base> {
    type LinkMeta: Send;
    fn init(self, base: Base) -> Self::LinkMeta;
}

/// One row of link input, split into the three parts that are collected separately.
pub struct InsertManyInit<PreOperation, OnInsert, SqlStatement> {
    pub pre_operation: PreOperation,
    pub on_insert: OnInsert,
    pub sql_statement: SqlStatement,
}

/// Per-row behavior of a link. Batch values are produced by the `*_from_iterator` traits.
pub trait InsertManyLinkMeta: Sized {
    type InputData: Send;
    type Output: Send;

    /// Split one row's [`InputData`](Self::InputData).
    fn init(
        &self,
        input: Self::InputData,
    ) -> InsertManyInit<Self::InitPreOperation, Self::InitOnInsert, Self::InitSqlStatement>;

    type InitPreOperation: Send;
    type InitOnInsert: Send;
    type InitSqlStatement: Send;

    /// Executable operation built from every row's pre-operation piece.
    type PreOperation: Send + crate::operations_v2::FallibleOperationOutput;
    /// Executable operation built from every row's post-operation piece.
    type PostOperation: Send + crate::operations_v2::FallibleOperationOutput;

    fn pre_operation_parts(
        &self,
        pre_operation_output: <Self::PreOperation as crate::operations_v2::FallibleOperationOutput>::Output,
    ) -> crate::operations_v2::insert_one::PreOperationParts<
        Self::PreOnInsert,
        Self::PreSqlStatement,
        Self::PreOutput,
    >;

    type PreOnInsert: Clone + Send;
    type PreSqlStatement: Clone + Send;
    type PreOutput: Clone + Send;

    fn identifiers(&self) -> Self::Identifiers;
    type Identifiers: Send;

    fn from_row(&self) -> Self::FromRow;
    type FromRow: crate::from_row::FromRowData + Send;

    /// Split one row after the insert statement.
    fn sql_statement_parts(
        &self,
        init_sql_statement: Self::InitSqlStatement,
        pre_sql_statement: Self::PreSqlStatement,
        from_row_data: <Self::FromRow as crate::from_row::FromRowData>::RData,
    ) -> crate::operations_v2::insert_one::SqlStatmentParts<
        Self::PostOperationItem,
        Self::SqlOutputItem,
    >;

    type PostOperationItem: Send;
    type SqlOutputItem: Send;

    /// One row of link output, before the batch is collected.
    fn output(
        &self,
        pre_output: Self::PreOutput,
        sql_output: Self::SqlOutputItem,
        post_output: <Self::PostOperation as crate::operations_v2::FallibleOperationOutput>::Output,
    ) -> Self::Output;
}

/// Collected pre-operation pieces become one [`PreOperation`](InsertManyLinkMeta::PreOperation).
pub trait InsertManyPreOperationFromIterator<Collected>: InsertManyLinkMeta {
    fn pre_operation_from_iterator(&self, collected: Collected) -> Self::PreOperation;
}

/// Collected on-insert pieces stay in the container shape.
pub trait InsertManyOnInsertFromIterator<Collected>: InsertManyLinkMeta {
    type OnInsert: IntoIterator<Item = Self::InitOnInsert> + Send;
    fn on_insert_from_iterator(&self, collected: Collected) -> Self::OnInsert;
}

/// Collected sql-statement pieces stay in the container shape.
pub trait InsertManySqlStatementFromIterator<Collected>: InsertManyLinkMeta {
    type SqlStatement: IntoIterator<Item = Self::InitSqlStatement> + Send;
    fn sql_statement_from_iterator(&self, collected: Collected) -> Self::SqlStatement;
}

/// Collected post-operation pieces become one [`PostOperation`](InsertManyLinkMeta::PostOperation).
pub trait InsertManyPostOperationFromIterator<Collected>: InsertManyLinkMeta {
    fn post_operation_from_iterator(&self, collected: Collected) -> Self::PostOperation;
}

/// Collected per-row outputs, still in the input container's shape.
pub trait InsertManyOutputFromIterator<Collected>: InsertManyLinkMeta {
    type CollectedOutput: IntoIterator<Item = Self::Output> + Send;
    fn output_from_iterator(&self, collected: Collected) -> Self::CollectedOutput;
}

pub mod iterator_meta {
    /// Information about the shape of the container type, with `Collect` you
    /// can collect the iterator while preserving the shape of the container type
    ///
    /// example:
    /// ```ignore
    /// let data: [char; 3] = ['a', 'b', 'c'];
    /// let iterator_meta = data.iter_meta();
    ///
    /// let after_iteration: [usize; 3] = iterator_meta.collect_from_meta(data.iter().map(|x| x.len_utf8()));
    /// ```
    pub trait IteratorMeta {
        type IterMeta;
        type Item;
        fn iter_meta(&self) -> Self::IterMeta;
    }

    /// Collect the iterator with a specific shape
    ///
    /// example:
    /// ```ignore
    /// let data: [char; 3] = ['a', 'b', 'c'];
    /// let iterator_meta = data.iter_meta();
    ///
    /// let after_iteration: [usize; 3] = iterator_meta.collect_from_meta(data.iter().map(|x| x.len_utf8()));
    /// ```
    pub trait CollectFromMeta {
        type Output<Item>;
        fn collect_from_meta<Iter: Iterator>(self, items: Iter) -> Self::Output<Iter::Item>;
    }

    /// Meta for [T; N]
    #[derive(Clone, Copy, Debug)]
    pub struct ArrayMeta<const N: usize>;

    impl<T, const N: usize> IteratorMeta for [T; N] {
        type IterMeta = ArrayMeta<N>;
        type Item = T;
        fn iter_meta(&self) -> Self::IterMeta {
            ArrayMeta
        }
    }

    impl<const N: usize> CollectFromMeta for ArrayMeta<N> {
        type Output<I> = [I; N];
        fn collect_from_meta<I: Iterator>(self, mut items: I) -> Self::Output<I::Item> {
            [(); N].map(|_| items.next().expect("bug: ArrayMeta used to collect the iterator doesn't belong to the array's type"))
        }
    }

    /// Meta for Vec<T>
    #[derive(Clone, Copy, Debug)]
    pub struct VecMeta {
        pub len: usize,
    }

    impl<T> IteratorMeta for Vec<T> {
        type IterMeta = VecMeta;
        type Item = T;
        fn iter_meta(&self) -> Self::IterMeta {
            VecMeta { len: self.len() }
        }
    }

    impl CollectFromMeta for VecMeta {
        type Output<I> = Vec<I>;
        fn collect_from_meta<I: Iterator>(self, mut items: I) -> Self::Output<I::Item> {
            let mut output = Vec::with_capacity(self.len);
            for _ in 0..self.len {
                output.push(items.next().unwrap());
            }
            output
        }
    }
}

mod insert_id_many {
    use std::marker::PhantomData;

    use crate::{
        collections::CollectionMeta,
        from_row::FromRowData,
        operations_v2::{
            FallibleOperationOutput,
            insert_many::{
                InsertManyInit, InsertManyLink, InsertManyLinkMeta, InsertManyOnInsertFromIterator,
                InsertManyOutputFromIterator, InsertManyPostOperationFromIterator,
                InsertManyPreOperationFromIterator, InsertManySqlStatementFromIterator,
            },
            insert_one::{
                PreOperationParts, SqlStatmentParts,
                insert_id_link::{InsertId, InsertIdMeta},
            },
        },
        valid_syntax::OnInsert,
    };

    impl<Base> InsertManyLink<Base> for InsertId<PhantomData<Base::InputIdData>>
    where
        Base: CollectionMeta,
        Base::IdMeta: Send + for<'a> OnInsert<'a, InputData = Base::InputIdData>,
        Base::InputIdData: Send,
    {
        type LinkMeta = InsertIdMeta<Base::IdMeta>;

        fn init(self, base: Base) -> Self::LinkMeta {
            InsertIdMeta::new(base.id_meta())
        }
    }

    impl<IdMeta, Input> InsertManyLinkMeta for InsertIdMeta<IdMeta>
    where
        IdMeta: Send + for<'a> OnInsert<'a, InputData = Input>,
        Input: Send,
    {
        type InputData = InsertId<Input>;
        type Output = ();

        fn init(&self, input: Self::InputData) -> InsertManyInit<(), Input, ()> {
            InsertManyInit {
                pre_operation: (),
                on_insert: input.0,
                sql_statement: (),
            }
        }

        type InitPreOperation = ();
        type InitOnInsert = Input;
        type InitSqlStatement = ();

        type PreOperation = ();
        type PostOperation = ();

        fn pre_operation_parts(
            &self,
            _: <Self::PreOperation as FallibleOperationOutput>::Output,
        ) -> PreOperationParts<(), (), ()> {
            PreOperationParts {
                pre_operation__on_insert: (),
                pre_operation__sql_statement: (),
                pre_operation__output: (),
            }
        }

        type PreOnInsert = ();
        type PreSqlStatement = ();
        type PreOutput = ();

        fn identifiers(&self) -> Self::Identifiers {}
        type Identifiers = ();

        fn from_row(&self) -> Self::FromRow {}
        type FromRow = ();

        fn sql_statement_parts(
            &self,
            _: Self::InitSqlStatement,
            _: Self::PreSqlStatement,
            _: <Self::FromRow as FromRowData>::RData,
        ) -> SqlStatmentParts<(), ()> {
            SqlStatmentParts {
                post_operation: (),
                sql_statement__output: (),
            }
        }

        type PostOperationItem = ();
        type SqlOutputItem = ();

        fn output(
            &self,
            _: Self::PreOutput,
            _: Self::SqlOutputItem,
            _: <Self::PostOperation as FallibleOperationOutput>::Output,
        ) -> Self::Output {
        }
    }

    impl<IdMeta, Input, Collected> InsertManyPreOperationFromIterator<Collected>
        for InsertIdMeta<IdMeta>
    where
        IdMeta: Send + for<'a> OnInsert<'a, InputData = Input>,
        Input: Send,
    {
        fn pre_operation_from_iterator(&self, _: Collected) -> Self::PreOperation {}
    }

    impl<IdMeta, Input, Collected> InsertManyPostOperationFromIterator<Collected>
        for InsertIdMeta<IdMeta>
    where
        IdMeta: Send + for<'a> OnInsert<'a, InputData = Input>,
        Input: Send,
    {
        fn post_operation_from_iterator(&self, _: Collected) -> Self::PostOperation {}
    }

    impl<IdMeta, Input, const N: usize> InsertManyOnInsertFromIterator<[Input; N]>
        for InsertIdMeta<IdMeta>
    where
        IdMeta: Send + for<'a> OnInsert<'a, InputData = Input>,
        Input: Send,
    {
        type OnInsert = [Input; N];

        fn on_insert_from_iterator(&self, collected: [Input; N]) -> Self::OnInsert {
            collected
        }
    }

    impl<IdMeta, Input> InsertManyOnInsertFromIterator<Vec<Input>> for InsertIdMeta<IdMeta>
    where
        IdMeta: Send + for<'a> OnInsert<'a, InputData = Input>,
        Input: Send,
    {
        type OnInsert = Vec<Input>;

        fn on_insert_from_iterator(&self, collected: Vec<Input>) -> Self::OnInsert {
            collected
        }
    }

    impl<IdMeta, Input, const N: usize> InsertManySqlStatementFromIterator<[(); N]>
        for InsertIdMeta<IdMeta>
    where
        IdMeta: Send + for<'a> OnInsert<'a, InputData = Input>,
        Input: Send,
    {
        type SqlStatement = [(); N];

        fn sql_statement_from_iterator(&self, collected: [(); N]) -> Self::SqlStatement {
            collected
        }
    }

    impl<IdMeta, Input> InsertManySqlStatementFromIterator<Vec<()>> for InsertIdMeta<IdMeta>
    where
        IdMeta: Send + for<'a> OnInsert<'a, InputData = Input>,
        Input: Send,
    {
        type SqlStatement = Vec<()>;

        fn sql_statement_from_iterator(&self, collected: Vec<()>) -> Self::SqlStatement {
            collected
        }
    }

    impl<IdMeta, Input, const N: usize> InsertManyOutputFromIterator<[(); N]> for InsertIdMeta<IdMeta>
    where
        IdMeta: Send + for<'a> OnInsert<'a, InputData = Input>,
        Input: Send,
    {
        type CollectedOutput = [(); N];

        fn output_from_iterator(&self, collected: [(); N]) -> Self::CollectedOutput {
            collected
        }
    }

    impl<IdMeta, Input> InsertManyOutputFromIterator<Vec<()>> for InsertIdMeta<IdMeta>
    where
        IdMeta: Send + for<'a> OnInsert<'a, InputData = Input>,
        Input: Send,
    {
        type CollectedOutput = Vec<()>;

        fn output_from_iterator(&self, collected: Vec<()>) -> Self::CollectedOutput {
            collected
        }
    }
}

mod impl_operation_for_insert_many_ {
    use super::{
        InsertMany, InsertManyLink, InsertManyLinkMeta, InsertManyOnInsertFromIterator,
        InsertManyOutputFromIterator, InsertManyPostOperationFromIterator,
        InsertManyPreOperationFromIterator, InsertManySqlStatementFromIterator,
        iterator_meta::{CollectFromMeta, IteratorMeta},
    };
    use crate::{
        collections::CollectionMeta,
        execute::Executable,
        extend_sqlx::{
            DatabaseStatementBuilder,
            database_error::{ErrorHandler, InfalibilityStrategy},
        },
        fix_executor::ExecutorTrait,
        from_row::{FromRow, FromRowData},
        operations::LinkedOutput,
        operations_v2::{
            FallibleOperation, FallibleOperationOutput, insert_many::InsertEntity,
            insert_one::OnInsertParts,
        },
        sqlx_query_builder::{
            Expression,
            combinators::Nest,
            statements::insert_statement::{InsertManyValues, InsertStatement},
        },
        valid_syntax::{MembersExpressions, OnInsert, TableExpressions},
    };

    /// Row values in the same container shape as the insert input.
    struct ShapedRows<C: CollectFromMeta, T> {
        rows: C::Output<T>,
    }

    impl<C, T> IntoIterator for ShapedRows<C, T>
    where
        C: CollectFromMeta,
        C::Output<T>: IntoIterator<Item = T>,
    {
        type Item = T;
        type IntoIter = <C::Output<T> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.rows.into_iter()
        }
    }

    impl<Meta, PsuedoLink, Data, DataIteratorMeta, LinkMeta, LinkData, Infalibility>
        FallibleOperationOutput for InsertMany<Meta, PsuedoLink, Data, Infalibility>
    where
        Meta: CollectionMeta<OutputIdData: Send, OutputData: Send>,
        PsuedoLink: InsertManyLink<Meta, LinkMeta = LinkMeta>,
        LinkMeta: InsertManyLinkMeta<InputData = LinkData, Output: Send>,
        Data: IteratorMeta<
                IterMeta = DataIteratorMeta,
                Item = InsertEntity<<Meta as CollectionMeta>::InputData, LinkData>,
            >,
        DataIteratorMeta: Send + CollectFromMeta,
        Infalibility: Send + InfalibilityStrategy<Error: Send>,
        <DataIteratorMeta as CollectFromMeta>::Output<
            LinkedOutput<Meta::OutputIdData, Meta::OutputData, LinkMeta::Output>,
        >: Send,
    {
        type Output = DataIteratorMeta::Output<
            LinkedOutput<Meta::OutputIdData, Meta::OutputData, LinkMeta::Output>,
        >;

        type Error = Infalibility::Error;
    }

    impl<
        S,
        Meta,
        PsuedoLink,
        Data,
        DataIteratorMeta,
        LinkMeta,
        LinkData,
        LinkOutput,
        InitPreOperation,
        InitOnInsert,
        InitSqlStatement,
        PreOnInsert,
        PostOperationItem,
        Infalibility,
    > FallibleOperation<S> for InsertMany<Meta, PsuedoLink, Data, Infalibility>
    where
        Infalibility: Send + ErrorHandler<S, Error: Send>,
        S: DatabaseStatementBuilder + ExecutorTrait,
        Meta: CollectionMeta<IdMeta: for<'a> MembersExpressions<'a>>
            + for<'a> TableExpressions<'a>
            + for<'a> MembersExpressions<'a>
            + for<'a> OnInsert<'a, InputData = <Meta as CollectionMeta>::InputData>
            + Clone
            + Send,
        Meta::OutputData: Send,
        Meta::OutputIdData: Send,
        <Meta as CollectionMeta>::InputData: Send,
        Meta::IdMeta: Send + for<'r> FromRow<'r, S::Row, RData = Meta::OutputIdData>,
        Meta: for<'r> FromRow<'r, S::Row, RData = Meta::OutputData>,
        PsuedoLink: InsertManyLink<Meta, LinkMeta = LinkMeta> + Send,
        LinkData: Send,
        LinkOutput: Send,
        InitPreOperation: Send,
        InitOnInsert: Send,
        InitSqlStatement: Send,
        PreOnInsert: Clone + Send,
        PostOperationItem: Send,
        LinkMeta: Send
            + InsertManyLinkMeta<
                InputData = LinkData,
                Output = LinkOutput,
                InitPreOperation = InitPreOperation,
                InitOnInsert = InitOnInsert,
                InitSqlStatement = InitSqlStatement,
                PreOnInsert = PreOnInsert,
                PostOperationItem = PostOperationItem,
            > + for<'a> OnInsert<'a, InputData = OnInsertParts<InitOnInsert, PreOnInsert>>,
        for<'r> LinkMeta::FromRow: FromRow<'r, S::Row>,
        Data: Send
            + IntoIterator<Item = InsertEntity<<Meta as CollectionMeta>::InputData, LinkData>>
            + IteratorMeta<
                IterMeta = DataIteratorMeta,
                Item = InsertEntity<<Meta as CollectionMeta>::InputData, LinkData>,
            >,
        DataIteratorMeta: Copy + Send + CollectFromMeta,
        DataIteratorMeta::Output<LinkedOutput<Meta::OutputIdData, Meta::OutputData, LinkOutput>>:
            Send,
        DataIteratorMeta::Output<LinkMeta::SqlOutputItem>:
            IntoIterator<Item = LinkMeta::SqlOutputItem> + Send,
        LinkMeta::PreOperation: FallibleOperation<S, Error: Into<Self::Error>>,
        LinkMeta::PostOperation: FallibleOperation<S, Error: Into<Self::Error>>,
        <LinkMeta::PostOperation as FallibleOperationOutput>::Output: Clone,
        LinkMeta: InsertManyPreOperationFromIterator<DataIteratorMeta::Output<InitPreOperation>>,
        LinkMeta: InsertManyOnInsertFromIterator<DataIteratorMeta::Output<InitOnInsert>>,
        LinkMeta: InsertManySqlStatementFromIterator<DataIteratorMeta::Output<InitSqlStatement>>,
        LinkMeta: InsertManyPostOperationFromIterator<DataIteratorMeta::Output<PostOperationItem>>,
        LinkMeta: InsertManyOutputFromIterator<DataIteratorMeta::Output<LinkOutput>>,
        for<'a, 'q> InsertStatement<
            <Meta as TableExpressions<'a>>::PascalCase,
            (
                Nest<<Meta as OnInsert<'a>>::InsertIdentifiers>,
                Nest<<LinkMeta as OnInsert<'a>>::InsertIdentifiers>,
            ),
            InsertManyValues<
                ShapedRows<
                    DataIteratorMeta,
                    (
                        Nest<<Meta as OnInsert<'a>>::InsertValues>,
                        Nest<<LinkMeta as OnInsert<'a>>::InsertValues>,
                    ),
                >,
            >,
            (
                Nest<<Meta::IdMeta as MembersExpressions<'a>>::Identifiers>,
                Nest<<Meta as MembersExpressions<'a>>::Identifiers>,
                Nest<LinkMeta::Identifiers>,
            ),
        >: Expression<'q, S>,
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
                let link_meta = self.psuedo_link.init(self.meta.clone());
                let shape = self.data.iter_meta();

                let mut attributes = Vec::new();
                let mut pre_items = Vec::new();
                let mut on_insert_items = Vec::new();
                let mut sql_items = Vec::new();
                for row in self.data {
                    let init = link_meta.init(row.link);
                    attributes.push(row.attributes);
                    pre_items.push(init.pre_operation);
                    on_insert_items.push(init.on_insert);
                    sql_items.push(init.sql_statement);
                }
                let row_count = attributes.len();

                let pre_collected = shape.collect_from_meta(pre_items.into_iter());
                let on_insert_collected = shape.collect_from_meta(on_insert_items.into_iter());
                let sql_collected = shape.collect_from_meta(sql_items.into_iter());

                let pre_operation = link_meta.pre_operation_from_iterator(pre_collected);
                let on_insert = link_meta.on_insert_from_iterator(on_insert_collected);
                let sql_statement = link_meta.sql_statement_from_iterator(sql_collected);

                let pre_operation_output = pre_operation
                    .exec_operation(conn)
                    .await
                    .map_err(Into::into)?;
                let pre_parts = link_meta.pre_operation_parts(pre_operation_output);

                let (sql_string, arguments) = {
                    let value_rows = ShapedRows {
                        rows: shape.collect_from_meta(attributes.into_iter().zip(on_insert).map(
                            |(attribute, on_insert_item)| {
                                (
                                    Nest(self.meta.on_insert_values(attribute)),
                                    Nest(link_meta.on_insert_values(OnInsertParts {
                                        from_init: on_insert_item,
                                        from_pre_operation:
                                            pre_parts.pre_operation__on_insert.clone(),
                                    })),
                                )
                            },
                        )),
                    };

                    Expression::<'_, S>::sql_statement(InsertStatement {
                        table_name: self.meta.pascal_case(),
                        identifiers: (
                            Nest(self.meta.on_insert_identifiers()),
                            Nest(link_meta.on_insert_identifiers()),
                        ),
                        values: InsertManyValues(value_rows),
                        returning: (
                            Nest(id_meta.identifiers()),
                            Nest(self.meta.identifiers()),
                            Nest(link_meta.identifiers()),
                        ),
                    })
                };

                let returned = <S as ExecutorTrait>::fetch_all(
                    conn,
                    Executable {
                        string: sql_string.as_str(),
                        arguments,
                    },
                )
                .await
                .map_err(|err| self.infalibility.handle_sqlx_error(err))?;

                if returned.len() != row_count {
                    panic!("bug: returning is specified");
                }

                let mut ids = Vec::with_capacity(row_count);
                let mut decoded_attributes = Vec::with_capacity(row_count);
                let mut post_items = Vec::with_capacity(row_count);
                let mut sql_output_items = Vec::with_capacity(row_count);
                for (row, init_sql) in returned.iter().zip(sql_statement) {
                    ids.push(
                        id_meta
                            .from_row_no_alias(row)
                            .expect("bug: id returning decode"),
                    );
                    decoded_attributes.push(
                        self.meta
                            .from_row_no_alias(row)
                            .expect("bug: attribute returning decode"),
                    );
                    let from_row_data = link_meta
                        .from_row()
                        .from_row_no_alias(row)
                        .expect("bug: link returning decode");
                    let sql_parts = link_meta.sql_statement_parts(
                        init_sql,
                        pre_parts.pre_operation__sql_statement.clone(),
                        from_row_data,
                    );
                    post_items.push(sql_parts.post_operation);
                    sql_output_items.push(sql_parts.sql_statement__output);
                }

                let post_collected = shape.collect_from_meta(post_items.into_iter());
                let sql_output_collected = shape.collect_from_meta(sql_output_items.into_iter());
                let post_operation = link_meta.post_operation_from_iterator(post_collected);
                let post_output = post_operation
                    .exec_operation(conn)
                    .await
                    .map_err(Into::into)?;

                let mut link_items = Vec::with_capacity(row_count);
                for sql_output in sql_output_collected {
                    link_items.push(link_meta.output(
                        pre_parts.pre_operation__output.clone(),
                        sql_output,
                        post_output.clone(),
                    ));
                }
                let link_collected = shape.collect_from_meta(link_items.into_iter());
                let links = link_meta.output_from_iterator(link_collected);

                let outputs = ids.into_iter().zip(decoded_attributes).zip(links).map(
                    |((id, attributes), links)| LinkedOutput {
                        id,
                        attributes,
                        links,
                    },
                );
                Ok(shape.collect_from_meta(outputs))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use sqlx::{Sqlite, SqliteConnection};

    use crate::collections::CollectionMeta;
    use crate::define_collection::collections_example::{Todo, TodoMeta};
    use crate::extend_sqlx::database_error::{ErrorHandler, PropagateStateErrors, StateError};
    use crate::from_row::FromRow;
    use crate::operations::LinkedOutput;
    use crate::operations_v2::FallibleOperation;
    use crate::operations_v2::insert_many::InsertEntity;
    use crate::operations_v2::insert_one::OnInsertParts;
    use crate::operations_v2::insert_one::insert_id_link::InsertId;

    use super::iterator_meta::{CollectFromMeta, IteratorMeta};
    use crate::valid_syntax::{MembersExpressions, OnInsert};
    use crate::{
        execute::Executable,
        extend_sqlx::InMemoryConnection,
        fix_executor::ExecutorTrait,
        sqlx_query_builder::{
            Expression,
            combinators::Nest,
            statements::insert_statement::{InsertManyValues, InsertStatement},
        },
        valid_syntax::TableExpressions,
    };

    use super::*;

    type InsertManyOperationResult = Result<[LinkedOutput<i64, Todo, ()>; 2], StateError>;
    type InsertManyOperation = InsertMany<
        TodoMeta,
        InsertId<PhantomData<i64>>,
        [InsertEntity<Todo, InsertId<i64>>; 2],
        PropagateStateErrors,
    >;

    #[tokio::test]
    async fn insert_many() {
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

        let from_body = operation_body(many([row("first", 20), row("second", 21)]), &mut conn)
            .await
            .unwrap();

        let from_interface =
            operation_interface(many([row("third", 22), row("fourth", 23)]), &mut conn)
                .await
                .unwrap();

        assert_eq!(from_body, [linked("first", 20), linked("second", 21)],);
        assert_eq!(from_interface, [linked("third", 22), linked("fourth", 23)],);
    }

    fn row(title: &str, id: i64) -> InsertEntity<Todo, InsertId<i64>> {
        InsertEntity {
            attributes: Todo {
                title: title.to_string(),
                done: false,
                description: Some(title.to_string()),
            },
            link: InsertId(id),
        }
    }

    fn linked(title: &str, id: i64) -> LinkedOutput<i64, Todo, ()> {
        LinkedOutput {
            id,
            attributes: Todo {
                title: title.to_string(),
                done: false,
                description: Some(title.to_string()),
            },
            links: (),
        }
    }

    fn many(data: [InsertEntity<Todo, InsertId<i64>>; 2]) -> InsertManyOperation {
        InsertMany {
            meta: TodoMeta,
            psuedo_link: InsertId(PhantomData::<i64>),
            data,
            infalibility: PropagateStateErrors,
        }
    }

    async fn operation_body(
        this: InsertManyOperation,
        conn: &mut SqliteConnection,
    ) -> InsertManyOperationResult {
        let id_meta = this.meta.id_meta();
        let link_meta = this.psuedo_link.init(this.meta.clone());
        let shape = this.data.iter_meta();

        let mut attributes = Vec::new();
        let mut pre_items = Vec::new();
        let mut on_insert_items = Vec::new();
        let mut sql_items = Vec::new();
        for row in this.data {
            let init = link_meta.init(row.link);
            attributes.push(row.attributes);
            pre_items.push(init.pre_operation);
            on_insert_items.push(init.on_insert);
            sql_items.push(init.sql_statement);
        }

        let pre_operation =
            link_meta.pre_operation_from_iterator(shape.collect_from_meta(pre_items.into_iter()));
        let on_insert =
            link_meta.on_insert_from_iterator(shape.collect_from_meta(on_insert_items.into_iter()));
        let sql_statement =
            link_meta.sql_statement_from_iterator(shape.collect_from_meta(sql_items.into_iter()));

        let pre_operation_output =
            FallibleOperation::<Sqlite>::exec_operation(pre_operation, conn).await?;
        let pre_parts = link_meta.pre_operation_parts(pre_operation_output);

        let value_rows = shape.collect_from_meta(attributes.into_iter().zip(on_insert).map(
            |(attribute, on_insert_item)| {
                (
                    Nest(this.meta.on_insert_values(attribute)),
                    Nest(link_meta.on_insert_values(OnInsertParts {
                        from_init: on_insert_item,
                        from_pre_operation: pre_parts.pre_operation__on_insert,
                    })),
                )
            },
        ));

        let (sql, arg) = Expression::<Sqlite>::sql_statement(InsertStatement {
            table_name: TableExpressions::pascal_case(&this.meta),
            identifiers: (
                Nest(this.meta.on_insert_identifiers()),
                Nest(link_meta.on_insert_identifiers()),
            ),
            values: InsertManyValues(value_rows),
            returning: (
                Nest(id_meta.identifiers()),
                Nest(this.meta.identifiers()),
                Nest(link_meta.identifiers()),
            ),
        });

        let returned = <Sqlite as ExecutorTrait>::fetch_all(
            conn,
            Executable {
                string: &sql,
                arguments: arg,
            },
        )
        .await
        .map_err(|err| this.infalibility.handle_sqlx_error(err))?;

        let mut ids = Vec::new();
        let mut decoded_attributes = Vec::new();
        let mut post_items = Vec::new();
        let mut sql_output_items = Vec::new();
        for (row, init_sql) in returned.iter().zip(sql_statement) {
            ids.push(
                id_meta
                    .from_row_no_alias(row)
                    .expect("bug: id returning decode"),
            );
            decoded_attributes.push(
                this.meta
                    .from_row_no_alias(row)
                    .expect("bug: attribute returning decode"),
            );
            let from_row_data = link_meta
                .from_row()
                .from_row_no_alias(row)
                .expect("bug: link returning decode");
            let sql_parts = link_meta.sql_statement_parts(
                init_sql,
                pre_parts.pre_operation__sql_statement,
                from_row_data,
            );
            post_items.push(sql_parts.post_operation);
            sql_output_items.push(sql_parts.sql_statement__output);
        }

        let post_operation =
            link_meta.post_operation_from_iterator(shape.collect_from_meta(post_items.into_iter()));
        let post_output = FallibleOperation::<Sqlite>::exec_operation(post_operation, conn).await?;
        let mut link_items = Vec::new();
        for sql_output in shape.collect_from_meta(sql_output_items.into_iter()) {
            link_items.push(link_meta.output(
                pre_parts.pre_operation__output,
                sql_output,
                post_output,
            ));
        }
        let links = link_meta.output_from_iterator(shape.collect_from_meta(link_items.into_iter()));

        Ok(
            shape.collect_from_meta(ids.into_iter().zip(decoded_attributes).zip(links).map(
                |((id, attributes), links)| LinkedOutput {
                    id,
                    attributes,
                    links,
                },
            )),
        )
    }

    async fn operation_interface(
        this: InsertManyOperation,
        conn: &mut SqliteConnection,
    ) -> InsertManyOperationResult {
        FallibleOperation::<Sqlite>::exec_operation(this, conn).await
    }
}
