use crate::sqlx_query_builder::SealExpression;

pub struct CreateTable<TableCondition, TableName, ColDefs> {
    pub table_condition: TableCondition,
    pub name: TableName,
    pub col_defs: ColDefs,
}

pub mod table_conditions {
    use crate::sqlx_query_builder::SealExpression;

    pub struct IfNotExist;

    impl SealExpression for IfNotExist {}

    mod impl_if_not_exist_for_sqlite {
        use sqlx::Sqlite;

        use crate::{
            extend_sqlx::DatabaseStatementBuilder,
            sqlx_query_builder::{
                Expression, StatementBuilder,
                statements::create_table_statement::table_conditions::IfNotExist,
            },
        };

        impl<'q> Expression<'q, Sqlite> for IfNotExist {
            fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
            where
                Sqlite: DatabaseStatementBuilder,
            {
                ctx.syntax("IF NOT EXISTS");
            }
        }
    }
}

impl<Header, Table, Columns> SealExpression for CreateTable<Header, Table, Columns> {}

pub struct ColumnDefinition<C>(pub C);

impl<C> SealExpression for ColumnDefinition<C> {}

pub struct NotNull;

impl SealExpression for NotNull {}

mod impl_create_table_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{
            Expression, Join, Prefixed, StatementBuilder,
            statements::create_table_statement::{ColumnDefinition, CreateTable, NotNull},
        },
    };

    impl<'q, TableCondition, Table, ColumnsAndConstraints> Expression<'q, Sqlite>
        for CreateTable<TableCondition, Table, ColumnsAndConstraints>
    where
        Prefixed<TableCondition>: Expression<'q, Sqlite>,
        Table: Expression<'q, Sqlite>,
        Join<ColumnsAndConstraints>: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("CREATE TABLE");
            Prefixed {
                prefix: " ",
                inner: self.table_condition,
            }
            .expression(ctx);

            ctx.syntax(" ");
            self.name.expression(ctx);
            ctx.syntax(" (");
            Join {
                start: "",
                separator: ", ",
                items: self.col_defs,
            }
            .expression(ctx);
            ctx.syntax(");");
        }
    }

    impl<'q, C> Expression<'q, Sqlite> for ColumnDefinition<C>
    where
        Join<C>: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            Join {
                start: "",
                separator: " ",
                items: self.0,
            }
            .expression(ctx);
        }
    }

    impl<'q> Expression<'q, Sqlite> for NotNull {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("NOT NULL");
        }
    }
}
