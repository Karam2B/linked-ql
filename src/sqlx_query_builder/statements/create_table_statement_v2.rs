use crate::sqlx_query_builder::SealExpression;

pub struct CreateTable<Init, TableName, ColDefs> {
    pub init: Init,
    pub name: TableName,
    pub col_defs: ColDefs,
}

pub mod expressions {
    #![allow(non_camel_case_types)]

    use crate::sqlx_query_builder::SealExpression;

    pub struct CreateTableInit;

    impl SealExpression for CreateTableInit {}

    pub struct CreateTableInitIfNotExist;

    impl SealExpression for CreateTableInitIfNotExist {}

    mod impl_create_table_init_for_sqlite {
        use sqlx::Sqlite;

        use crate::{
            extend_sqlx::DatabaseStatementBuilder,
            sqlx_query_builder::{
                statements::create_table_statement::expressions::{
                    CreateTableInit, CreateTableInitIfNotExist,
                },
                Expression, StatementBuilder,
            },
        };

        impl<'q> Expression<'q, Sqlite> for CreateTableInit {
            fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
            where
                Sqlite: DatabaseStatementBuilder,
            {
                ctx.syntax("CREATE TABLE");
            }
        }

        impl<'q> Expression<'q, Sqlite> for CreateTableInitIfNotExist {
            fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
            where
                Sqlite: DatabaseStatementBuilder,
            {
                ctx.syntax("CREATE TABLE IF NOT EXISTS");
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
            statements::create_table_statement::{ColumnDefinition, CreateTable, NotNull},
            Expression, Join, StatementBuilder,
        },
    };

    impl<'q, Header, Table, Columns> Expression<'q, Sqlite> for CreateTable<Header, Table, Columns>
    where
        Header: Expression<'q, Sqlite>,
        Table: Expression<'q, Sqlite>,
        Join<Columns>: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            self.init.expression(ctx);
            ctx.syntax(" ");
            self.name.expression(ctx);
            ctx.syntax(" ");
            ctx.syntax("(");
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
