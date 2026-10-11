use crate::sqlx_query_builder::SealExpression;

pub struct AddColumn<Table, ColDef> {
    pub table: Table,
    pub col_def: ColDef,
}

pub struct DropColumn<Table, Name> {
    pub table: Table,
    pub col_name: Name,
}

impl<Table, ColDef> SealExpression for AddColumn<Table, ColDef> {}

mod impl_add_column_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{
            statements::add_column_statement::AddColumn, Expression, StatementBuilder,
        },
    };

    impl<'q, Table, ColDef> Expression<'q, Sqlite> for AddColumn<Table, ColDef>
    where
        Table: Expression<'q, Sqlite>,
        ColDef: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("ALTER TABLE ");
            self.table.expression(ctx);
            ctx.syntax(" ADD COLUMN ");
            self.col_def.expression(ctx);
            ctx.syntax(";");
        }
    }
}
