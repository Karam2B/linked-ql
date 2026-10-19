use crate::sqlx_query_builder::SealExpression;

pub struct CreateTrigger<TriggerName, Lifetime, OperationName, OnTable, WhenExpression, Statements>
{
    pub trigger_name: TriggerName,
    pub temp: bool,
    pub if_not_exists: bool,
    pub lifetime: Lifetime,
    pub operation_name: OperationName,
    pub on_table: OnTable,
    pub for_each_row: bool,
    pub when_expression: WhenExpression,
    pub statements: Statements,
}

impl<Tn, Lt, On, Ont, We, St> SealExpression for CreateTrigger<Tn, Lt, On, Ont, We, St> {}

pub struct TriggerLifetimeBefore;

impl SealExpression for TriggerLifetimeBefore {}

pub struct TriggerLifetimeAfter;

impl SealExpression for TriggerLifetimeAfter {}

pub struct TriggerLifetimeInsteadOf;

impl SealExpression for TriggerLifetimeInsteadOf {}

pub struct TriggerOperationNameInsert;

impl SealExpression for TriggerOperationNameInsert {}

pub struct TriggerOperationNameUpdate<OfColumns>(pub OfColumns);

impl<OfColumns> SealExpression for TriggerOperationNameUpdate<OfColumns> {}

mod impl_create_trigger_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{
            Expression, OptionalExpression, StatementBuilder,
            combinators::Join,
            statements::create_trigger::{
                CreateTrigger, TriggerLifetimeAfter, TriggerLifetimeBefore,
                TriggerLifetimeInsteadOf, TriggerOperationNameInsert, TriggerOperationNameUpdate,
            },
        },
    };

    impl<'q, TriggerName, Lifetime, OperationName, OnTable, WhenExpression, Statements>
        Expression<'q, Sqlite>
        for CreateTrigger<TriggerName, Lifetime, OperationName, OnTable, WhenExpression, Statements>
    where
        TriggerName: Expression<'q, Sqlite>,
        Lifetime: Expression<'q, Sqlite>,
        OperationName: Expression<'q, Sqlite>,
        OnTable: Expression<'q, Sqlite>,
        WhenExpression: OptionalExpression,
        Join<WhenExpression>: Expression<'q, Sqlite>,
        Join<Statements>: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("CREATE ");
            if self.temp {
                ctx.syntax("TEMP ");
            }
            if self.if_not_exists {
                ctx.syntax("IF NOT EXISTS ");
            }
            ctx.syntax("TRIGGER ");
            self.trigger_name.expression(ctx);
            ctx.syntax(" ");
            self.lifetime.expression(ctx);
            ctx.syntax(" ");
            self.operation_name.expression(ctx);
            ctx.syntax(" ON ");
            self.on_table.expression(ctx);
            if self.for_each_row {
                ctx.syntax(" FOR EACH ROW ");
            }
            Join {
                start: " WHEN ",
                separator: " ",
                items: self.when_expression,
            }
            .expression(ctx);
            ctx.syntax(" BEGIN ");
            Join {
                start: "",
                separator: ";",
                items: self.statements,
            }
            .expression(ctx);
            ctx.syntax(" END;");
        }
    }

    impl<'q> Expression<'q, Sqlite> for TriggerLifetimeBefore {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("BEFORE");
        }
    }

    impl<'q> Expression<'q, Sqlite> for TriggerLifetimeAfter {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("AFTER");
        }
    }

    impl<'q> Expression<'q, Sqlite> for TriggerLifetimeInsteadOf {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("INSTEAD OF");
        }
    }

    impl<'q> Expression<'q, Sqlite> for TriggerOperationNameInsert {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("INSERT");
        }
    }

    impl<'q, OfColumns> Expression<'q, Sqlite> for TriggerOperationNameUpdate<OfColumns>
    where
        Join<OfColumns>: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("UPDATE");
            Join {
                start: " OF ",
                separator: ", ",
                items: self.0,
            }
            .expression(ctx);
        }
    }
}
