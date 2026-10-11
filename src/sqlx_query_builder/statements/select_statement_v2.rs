use crate::sqlx_query_builder::SealExpression;

pub struct SelectStatement<SelectItems, From, Joins, Wheres, GroupBy, Order, Limit> {
    pub select_items: SelectItems,
    pub from: From,
    pub joins: Joins,
    pub wheres: Wheres,
    pub group_by: GroupBy,
    pub order: Order,
    pub limit: Limit,
}

impl<SelectItems, From, Joins, GroupBy, Wheres, Limit, Order> SealExpression
    for SelectStatement<SelectItems, From, Joins, GroupBy, Wheres, Limit, Order>
{
}

pub struct SelectStatementV2<SelectItems, From, Joins, Wheres, GroupBy, Order, Limit> {
    pub select_items: SelectItems,
    pub from: From,
    pub joins: Joins,
    pub wheres: Wheres,
    pub group_by: GroupBy,
    pub order: Order,
    pub limit: Limit,
}

impl<SelectItems, From, Joins, Wheres, GroupBy, Order, Limit> SealExpression
    for SelectStatementV2<SelectItems, From, Joins, Wheres, GroupBy, Order, Limit>
{
}

pub struct LimitClause<Limit> {
    pub limit: Limit,
}

impl<Limit> SealExpression for LimitClause<Limit> {}

pub struct FromClause<From> {
    pub from: From,
}

impl<From> SealExpression for FromClause<From> {}

pub struct Aliased<Alias, Aliased> {
    pub aliased: Aliased,
    pub alias: Alias,
}

impl<A, Ad> SealExpression for Aliased<A, Ad> {}

mod impl_select_statement_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{
            statements::select_statement::{
                Aliased, FromClause, LimitClause, SelectStatement, SelectStatementV2,
            },
            Expression, Join, OptionalExpression, Prefixed, StatementBuilder,
        },
    };

    impl<'q, SelectItems, From, Joins, Wheres, Limit, Order, GroupBy> Expression<'q, Sqlite>
        for SelectStatement<SelectItems, From, Joins, Wheres, GroupBy, Order, Limit>
    where
        Join<SelectItems>: Expression<'q, Sqlite>,
        SelectItems: OptionalExpression,
        From: Expression<'q, Sqlite>,
        Join<Joins>: Expression<'q, Sqlite>,
        Join<GroupBy>: Expression<'q, Sqlite>,
        Join<Wheres>: Expression<'q, Sqlite>,
        Prefixed<Limit>: Expression<'q, Sqlite>,
        Join<Order>: Expression<'q, Sqlite>,
    {
        #[track_caller]
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("SELECT ");
            let join_select_items = Join {
                start: "",
                separator: ", ",
                items: self.select_items,
            };

            if join_select_items.items.is_expression_present() {
                join_select_items.expression(ctx);
            } else {
                panic!("empty select item")
            }

            ctx.syntax(" FROM ");
            self.from.expression(ctx);
            Join {
                start: " ",
                separator: ", ",
                items: self.joins,
            }
            .expression(ctx);
            Join {
                start: " WHERE ",
                separator: " AND ",
                items: self.wheres,
            }
            .expression(ctx);
            Join {
                start: " GROUP BY ",
                separator: ", ",
                items: self.group_by,
            }
            .expression(ctx);
            Join {
                start: " ORDER BY ",
                separator: ", ",
                items: self.order,
            }
            .expression(ctx);
            Prefixed {
                prefix: " LIMIT ",
                inner: self.limit,
            }
            .expression(ctx);
            ctx.syntax(";");
        }
    }

    impl<'q, SelectItems, From, Joins, Wheres, Limit, Order, GroupBy> Expression<'q, Sqlite>
        for SelectStatementV2<SelectItems, From, Joins, Wheres, GroupBy, Order, Limit>
    where
        SelectItems: Expression<'q, Sqlite>,
        From: Expression<'q, Sqlite>,
        Limit: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("SELECT ");
            self.select_items.expression(ctx);
            self.from.expression(ctx);
            self.limit.expression(ctx);
            ctx.syntax(";");
        }
    }

    impl<'q, Limit> Expression<'q, Sqlite> for LimitClause<Limit>
    where
        Limit: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax(" LIMIT ");
            self.limit.expression(ctx);
        }
    }

    impl<'q, From> Expression<'q, Sqlite> for FromClause<From>
    where
        From: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax(" FROM ");
            self.from.expression(ctx);
        }
    }

    impl<'q, A, Ad> Expression<'q, Sqlite> for Aliased<A, Ad>
    where
        A: Expression<'q, Sqlite>,
        Ad: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            self.aliased.expression(ctx);
            ctx.syntax(" AS ");
            self.alias.expression(ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;

    use crate::sqlx_query_builder::{
        statements::select_statement::{Aliased, FromClause, LimitClause, SelectStatementV2},
        Bind, Expression,
    };

    #[test]
    fn test_select_statement_v2() {
        let alias = String::from("value");

        let stmt = SelectStatementV2 {
            select_items: Aliased {
                aliased: Bind(1),
                alias: alias.as_str(),
            },
            from: FromClause { from: "table" },
            joins: (),
            wheres: (),
            order: (),
            limit: LimitClause { limit: Bind(3) },
            group_by: (),
        };

        let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(stmt);
        assert_eq!(stmt, r#"SELECT $1 AS "value" FROM "table" LIMIT $2;"#);
    }
}
