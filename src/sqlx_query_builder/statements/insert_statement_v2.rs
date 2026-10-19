use crate::sqlx_query_builder::SealExpression;

/// Represents an INSERT statement.
///
/// there are two variants of using this, depending on how you wrap "Values":
/// - InsertOne<V>: insert one row
///         where Join<V>: Expression<'q, S>
///         example of V: (32, "John Doe"), vec![E::String("John Doe"), E::Number(32)]
/// - InsertMany<V>: insert multiple rows
///         where V: IntoIterator, Join<V::Item>: Expression<'q, S>
///         example of V: vec!((32, "John Doe"), (33, "Jane Doe"))
pub struct InsertStatement<TableName, Identifiers, Values, Returning> {
    pub table_name: TableName,
    pub identifiers: Identifiers,
    pub values: Values,
    pub returning: Returning,
}

impl<TableName, Identifiers, Values, Returning> SealExpression
    for InsertStatement<TableName, Identifiers, Values, Returning>
{
}

pub struct InsertOneValue<T>(pub T);

pub struct InsertManyValues<T>(pub T);

impl<T> SealExpression for InsertManyValues<T> {}
impl<T> SealExpression for InsertOneValue<T> {}

mod impl_insert_statement_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{
            Expression, OptionalExpression, StatementBuilder,
            combinators::Join,
            statements::insert_statement::{InsertManyValues, InsertOneValue, InsertStatement},
        },
    };

    impl<'q, ManyValues> Expression<'q, Sqlite> for InsertManyValues<ManyValues>
    where
        ManyValues: IntoIterator,
        Join<ManyValues::Item>: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            let mut iter = self.0.into_iter();

            let first = iter
                .next()
                .expect("should have at least one item to insert for a valid insert statement");

            Join {
                start: "(",
                separator: ", ",
                items: first,
            }
            .expression(ctx);
            ctx.syntax(")");

            for each in iter {
                ctx.syntax(", ");
                Join {
                    start: "(",
                    separator: ", ",
                    items: each,
                }
                .expression(ctx);
                ctx.syntax(")");
            }
        }
    }

    impl<'q, OneValue> Expression<'q, Sqlite> for InsertOneValue<OneValue>
    where
        OneValue: OptionalExpression,
        Join<OneValue>: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            if self.0.is_expression_present() {
                Join {
                    start: "(",
                    separator: ", ",
                    items: self.0,
                }
                .expression(ctx);
                ctx.syntax(")");
            } else {
                panic!("insert_statment with empty values")
            }
        }
    }

    impl<'q, TableName, Identifiers, Values, Returning> Expression<'q, Sqlite>
        for InsertStatement<TableName, Identifiers, Values, Returning>
    where
        TableName: Expression<'q, Sqlite>,
        Identifiers: OptionalExpression,
        Join<Identifiers>: Expression<'q, Sqlite>,
        Values: Expression<'q, Sqlite>,
        Returning: OptionalExpression,
        Join<Returning>: Expression<'q, Sqlite>,
    {
        #[track_caller]
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            if !self.identifiers.is_expression_present() {
                panic!("insert_statment with empty values")
            }
            ctx.syntax("INSERT INTO ");
            self.table_name.expression(ctx);
            ctx.syntax(" ");
            if self.identifiers.is_expression_present() {
                Join {
                    start: "(",
                    separator: ", ",
                    items: self.identifiers,
                }
                .expression(ctx);
                ctx.syntax(")");
            } else {
                panic!("insert_statment with empty identifiers")
            }

            ctx.syntax(" VALUES ");
            self.values.expression(ctx);

            Join {
                start: " RETURNING ",
                separator: ", ",
                items: self.returning,
            }
            .expression(ctx);

            ctx.syntax(";");
        }
    }
}
