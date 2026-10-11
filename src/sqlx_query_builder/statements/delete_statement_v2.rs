use crate::sqlx_query_builder::SealExpression;

pub struct DeleteStatement<TableName, Wheres, Returning> {
    pub table_name: TableName,
    pub wheres: Wheres,
    pub returning: Returning,
}

impl<T, W, R> SealExpression for DeleteStatement<T, W, R> {}

mod impl_delete_statement_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{
            statements::delete_statement::DeleteStatement, Expression, Join, StatementBuilder,
        },
    };

    impl<'q, TableName, Wheres, Returning> Expression<'q, Sqlite>
        for DeleteStatement<TableName, Wheres, Returning>
    where
        TableName: Expression<'q, Sqlite>,
        Join<Wheres>: Expression<'q, Sqlite>,
        Join<Returning>: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("DELETE FROM ");
            self.table_name.expression(ctx);
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
