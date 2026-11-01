//! Operation for inserting many records in one statement.
//!
//! Each row carries its own link value. The link follows the same flow as
//! [`InsertOne`](crate::operations_v2::insert_one::InsertOne): `init`, a
//! fallible pre-operation, `OnInsert` values, the insert, returning decode,
//! a fallible post-operation, then `output`. Column lists come from the first
//! row's link meta. Every row uses that same link type.

use crate::{collections::CollectionMeta, operations_v2::insert_one::InsertOneLink};

pub struct InsertMany<Meta: CollectionMeta, Link, Infalibility> {
    pub meta: Meta,
    pub rows: Vec<InsertEntity<<Meta as CollectionMeta>::InputData, Link>>,
    pub infalibility: Infalibility,
}

pub struct InsertEntity<Attributes, Link> {
    pub attributes: Attributes,
    pub link: Link,
}

mod impl_operation_for_insert_many {
    use super::*;
    use crate::execute::Executable;
    use crate::extend_sqlx::DatabaseStatementBuilder;
    use crate::extend_sqlx::database_error::{ErrorHandler, ErrorType};
    use crate::fix_executor::ExecutorTrait;
    use crate::from_row::FromRow;
    use crate::operations::LinkedOutput;
    use crate::operations_v2::insert_one::{InsertOneLinkMeta, InsertParts, OnInsertParts};
    use crate::operations_v2::{FallibleOperation, FallibleOperationOutput};
    use crate::sqlx_query_builder::Expression;
    use crate::sqlx_query_builder::combinators::Nest;
    use crate::sqlx_query_builder::statements::insert_statement::{
        InsertManyValues, InsertStatement,
    };
    use crate::valid_syntax::{MembersExpressions, OnInsert, TableExpressions};

    impl<Meta, Link, LinkMeta, Fallibility> FallibleOperationOutput
        for InsertMany<Meta, Link, Fallibility>
    where
        Fallibility: ErrorType<Error: Send>,
        Link: InsertOneLink<Meta, LinkMeta = LinkMeta>,
        LinkMeta: InsertOneLinkMeta<Output: Send>,
        Meta: CollectionMeta<OutputData: Send, OutputIdData: Send>,
    {
        type Output = Vec<LinkedOutput<Meta::OutputIdData, Meta::OutputData, LinkMeta::Output>>;
        type Error = Fallibility::Error;
    }

    impl<S, Meta, Link, LinkMeta, Fallibility> FallibleOperation<S>
        for InsertMany<Meta, Link, Fallibility>
    where
        Fallibility: Send + ErrorHandler<S, Error: Send>,
        S: DatabaseStatementBuilder + ExecutorTrait,
        Link: InsertOneLink<Meta, LinkMeta = LinkMeta> + Send,
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
            + Clone
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
            InsertManyValues<
                Vec<(
                    Nest<<Meta as OnInsert<'a>>::InsertValues>,
                    Nest<<LinkMeta as OnInsert<'a>>::InsertValues>,
                )>,
            >,
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
                if self.rows.is_empty() {
                    return Ok(Vec::new());
                }

                let id_meta = self.meta.id_meta();
                let mut link_metas = Vec::with_capacity(self.rows.len());
                let mut attributes = Vec::with_capacity(self.rows.len());
                let mut init_on_inserts = Vec::with_capacity(self.rows.len());
                let mut init_sql_statements = Vec::with_capacity(self.rows.len());
                let mut pre_on_inserts = Vec::with_capacity(self.rows.len());
                let mut pre_sql_statements = Vec::with_capacity(self.rows.len());
                let mut pre_outputs = Vec::with_capacity(self.rows.len());

                for row in self.rows {
                    let InsertParts {
                        link_meta,
                        pre_operation,
                        init_on__insert,
                        init_sql__statement,
                    } = row.link.init(self.meta.clone());

                    let pre_operation_output = pre_operation
                        .exec_operation(conn)
                        .await
                        .map_err(Into::into)?;
                    let pre_operation_parts = link_meta.pre_operation_parts(pre_operation_output);

                    link_metas.push(link_meta);
                    attributes.push(row.attributes);
                    init_on_inserts.push(init_on__insert);
                    init_sql_statements.push(init_sql__statement);
                    pre_on_inserts.push(pre_operation_parts.pre_operation__on_insert);
                    pre_sql_statements.push(pre_operation_parts.pre_operation__sql_statement);
                    pre_outputs.push(pre_operation_parts.pre_operation__output);
                }

                let (sql_string, arguments) = Expression::<'_, S>::sql_statement(InsertStatement {
                    table_name: self.meta.pascal_case(),
                    identifiers: (
                        Nest(self.meta.on_insert_identifiers()),
                        Nest(link_metas[0].on_insert_identifiers()),
                    ),
                    values: InsertManyValues(
                        attributes
                            .into_iter()
                            .zip(init_on_inserts)
                            .zip(pre_on_inserts)
                            .zip(link_metas.iter())
                            .map(
                                |(
                                    ((attribute, init_on_insert), pre_operation_on_insert),
                                    link_meta,
                                )| {
                                    (
                                        Nest(self.meta.on_insert_values(attribute)),
                                        Nest(link_meta.on_insert_values(OnInsertParts {
                                            from_init: init_on_insert,
                                            from_pre_operation: pre_operation_on_insert,
                                        })),
                                    )
                                },
                            )
                            .collect::<Vec<_>>(),
                    ),
                    returning: (
                        Nest(id_meta.identifiers()),
                        Nest(self.meta.identifiers()),
                        Nest(link_metas[0].indentifiers()),
                    ),
                });

                let returned = <S as ExecutorTrait>::fetch_all(
                    conn,
                    Executable {
                        string: sql_string.as_str(),
                        arguments,
                    },
                )
                .await
                .map_err(|err| self.infalibility.handle_sqlx_error(err))?;

                if returned.len() != link_metas.len() {
                    panic!("bug: returning is specified");
                }

                let mut outputs = Vec::with_capacity(returned.len());
                for (row, (((link_meta, init_sql_statement), pre_sql), pre_output)) in
                    returned.into_iter().zip(
                        link_metas
                            .into_iter()
                            .zip(init_sql_statements)
                            .zip(pre_sql_statements)
                            .zip(pre_outputs),
                    )
                {
                    let id = id_meta
                        .from_row_no_alias(&row)
                        .expect("bug: id returning decode");
                    let decoded_attributes = self
                        .meta
                        .from_row_no_alias(&row)
                        .expect("bug: attribute returning decode");
                    let from_row_data = link_meta
                        .from_row()
                        .from_row_no_alias(&row)
                        .expect("bug: link returning decode");
                    let sql_parts =
                        link_meta.sql_statement_parts(init_sql_statement, pre_sql, from_row_data);
                    let post_operation_output = sql_parts
                        .post_operation
                        .exec_operation(conn)
                        .await
                        .map_err(Into::into)?;
                    let links = link_meta.output(
                        pre_output,
                        sql_parts.sql_statement__output,
                        post_operation_output,
                    );
                    outputs.push(LinkedOutput {
                        id,
                        attributes: decoded_attributes,
                        links,
                    });
                }

                Ok(outputs)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Sqlite, SqliteConnection};

    use crate::define_collection::collections_example::{Todo, TodoMeta};
    use crate::extend_sqlx::database_error::{ErrorHandler, PropagateStateErrors, StateError};
    use crate::from_row::FromRow;
    use crate::operations::LinkedOutput;
    use crate::operations_v2::FallibleOperation;
    use crate::operations_v2::insert_one::insert_id_link::InsertId;
    use crate::operations_v2::insert_one::{InsertOneLinkMeta, InsertParts, OnInsertParts};
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

    type InsertManyOperationResult = Result<Vec<LinkedOutput<i64, Todo, ()>>, StateError>;

    fn todo(title: &str) -> Todo {
        Todo {
            title: title.to_string(),
            done: false,
            description: Some(title.to_string()),
        }
    }

    fn rows(
        first_id: i64,
        first_title: &str,
        second_title: &str,
    ) -> Vec<InsertEntity<Todo, InsertId<i64>>> {
        vec![
            InsertEntity {
                attributes: todo(first_title),
                link: InsertId(first_id),
            },
            InsertEntity {
                attributes: todo(second_title),
                link: InsertId(first_id + 1),
            },
        ]
    }

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

        let from_body = operation_body(
            InsertMany {
                meta: TodoMeta,
                rows: rows(10, "first", "second"),
                infalibility: PropagateStateErrors,
            },
            &mut conn,
        )
        .await
        .unwrap();

        let from_interface = operation_interface(
            InsertMany {
                meta: TodoMeta,
                rows: rows(12, "third", "fourth"),
                infalibility: PropagateStateErrors,
            },
            &mut conn,
        )
        .await
        .unwrap();

        assert_eq!(
            from_body,
            vec![
                LinkedOutput {
                    id: 10,
                    attributes: todo("first"),
                    links: (),
                },
                LinkedOutput {
                    id: 11,
                    attributes: todo("second"),
                    links: (),
                },
            ],
        );
        assert_eq!(
            from_interface,
            vec![
                LinkedOutput {
                    id: 12,
                    attributes: todo("third"),
                    links: (),
                },
                LinkedOutput {
                    id: 13,
                    attributes: todo("fourth"),
                    links: (),
                },
            ],
        );
    }

    async fn operation_body(
        this: InsertMany<TodoMeta, InsertId<i64>, PropagateStateErrors>,
        conn: &mut SqliteConnection,
    ) -> InsertManyOperationResult {
        if this.rows.is_empty() {
            return Ok(Vec::new());
        }

        let id_meta = this.meta.id_meta();
        let mut link_metas = Vec::with_capacity(this.rows.len());
        let mut attributes = Vec::with_capacity(this.rows.len());
        let mut init_on_inserts = Vec::with_capacity(this.rows.len());
        let mut init_sql_statements = Vec::with_capacity(this.rows.len());
        let mut pre_on_inserts = Vec::with_capacity(this.rows.len());
        let mut pre_sql_statements = Vec::with_capacity(this.rows.len());
        let mut pre_outputs = Vec::with_capacity(this.rows.len());

        for row in this.rows {
            let InsertParts {
                link_meta,
                pre_operation,
                init_on__insert,
                init_sql__statement,
            } = row.link.init(this.meta.clone());

            let pre_operation_output =
                FallibleOperation::<Sqlite>::exec_operation(pre_operation, conn).await?;
            let pre_operation_parts = link_meta.pre_operation_parts(pre_operation_output);

            link_metas.push(link_meta);
            attributes.push(row.attributes);
            init_on_inserts.push(init_on__insert);
            init_sql_statements.push(init_sql__statement);
            pre_on_inserts.push(pre_operation_parts.pre_operation__on_insert);
            pre_sql_statements.push(pre_operation_parts.pre_operation__sql_statement);
            pre_outputs.push(pre_operation_parts.pre_operation__output);
        }

        let (sql, arg) = Expression::<Sqlite>::sql_statement(InsertStatement {
            table_name: TableExpressions::pascal_case(&this.meta),
            identifiers: (
                Nest(this.meta.on_insert_identifiers()),
                Nest(link_metas[0].on_insert_identifiers()),
            ),
            values: InsertManyValues(
                attributes
                    .into_iter()
                    .zip(init_on_inserts)
                    .zip(pre_on_inserts)
                    .zip(link_metas.iter())
                    .map(
                        |(((attribute, init_on_insert), pre_operation_on_insert), link_meta)| {
                            (
                                Nest(this.meta.on_insert_values(attribute)),
                                Nest(link_meta.on_insert_values(OnInsertParts {
                                    from_init: init_on_insert,
                                    from_pre_operation: pre_operation_on_insert,
                                })),
                            )
                        },
                    )
                    .collect::<Vec<_>>(),
            ),
            returning: (
                Nest(id_meta.identifiers()),
                Nest(this.meta.identifiers()),
                Nest(link_metas[0].indentifiers()),
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

        if returned.len() != link_metas.len() {
            panic!("bug: returning is specified");
        }

        let mut outputs = Vec::with_capacity(returned.len());
        for (row, (((link_meta, init_sql_statement), pre_sql), pre_output)) in
            returned.into_iter().zip(
                link_metas
                    .into_iter()
                    .zip(init_sql_statements)
                    .zip(pre_sql_statements)
                    .zip(pre_outputs),
            )
        {
            let id = id_meta
                .from_row_no_alias(&row)
                .expect("bug: id returning decode");
            let decoded_attributes = this
                .meta
                .from_row_no_alias(&row)
                .expect("bug: attribute returning decode");
            let from_row_data = link_meta
                .from_row()
                .from_row_no_alias(&row)
                .expect("bug: link returning decode");
            let sql_parts =
                link_meta.sql_statement_parts(init_sql_statement, pre_sql, from_row_data);
            let post_operation_output =
                FallibleOperation::<Sqlite>::exec_operation(sql_parts.post_operation, conn).await?;
            let links = link_meta.output(
                pre_output,
                sql_parts.sql_statement__output,
                post_operation_output,
            );
            outputs.push(LinkedOutput {
                id,
                attributes: decoded_attributes,
                links,
            });
        }

        Ok(outputs)
    }

    async fn operation_interface(
        this: InsertMany<TodoMeta, InsertId<i64>, PropagateStateErrors>,
        conn: &mut SqliteConnection,
    ) -> InsertManyOperationResult {
        FallibleOperation::<Sqlite>::exec_operation(this, conn).await
    }
}
