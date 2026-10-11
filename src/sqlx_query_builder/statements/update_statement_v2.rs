use crate::sqlx_query_builder::SealExpression;

pub struct UpdateStatement<TableName, Values, Wheres, Returning> {
    pub table_name: TableName,
    pub values: Values,
    pub wheres: Wheres,
    pub returning: Returning,
}

impl<TableName, Values, Wheres, Returning> SealExpression
    for UpdateStatement<TableName, Values, Wheres, Returning>
{
}

mod impl_update_statement_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{
            statements::update_statement::UpdateStatement, Expression, Join, StatementBuilder,
        },
    };

    impl<'q, TableName, Values, Wheres, Returning> Expression<'q, Sqlite>
        for UpdateStatement<TableName, Values, Wheres, Returning>
    where
        TableName: Expression<'q, Sqlite>,
        Join<Values>: Expression<'q, Sqlite>,
        Join<Wheres>: Expression<'q, Sqlite>,
        Join<Returning>: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("UPDATE ");
            self.table_name.expression(ctx);
            ctx.syntax(" SET ");
            Join {
                start: "",
                separator: ", ",
                items: self.values,
            }
            .expression(ctx);
            Join {
                start: " WHERE ",
                separator: " AND ",
                items: self.wheres,
            }
            .expression(ctx);
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
