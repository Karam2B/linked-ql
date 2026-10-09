use std::marker::PhantomData;

use crate::{
    sqlx_query_builder::{OptionalExpression, SealExpression, SealRefExpression},
    update_mod::Update,
};

pub struct Bind<T>(pub T);

impl<T: Clone> Clone for Bind<T> {
    fn clone(&self) -> Self {
        Bind(self.0.clone())
    }
}

impl<T> SealExpression for Bind<T> {}

impl<T> OptionalExpression for Bind<T> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_bind {
    use sqlx::{Encode, Type};

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'q, T, S> Expression<'q, S> for super::Bind<T>
    where
        S: DatabaseStatementBuilder,
        T: 'q + Type<S> + Encode<'q, S>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
            S::impl_bind(self.0, &mut ctx.stmt, &mut ctx.info, &mut ctx.arg);
        }
    }
}

pub struct TypeAsSyntax<T>(pub PhantomData<T>);

impl<T> Clone for TypeAsSyntax<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<T> SealExpression for TypeAsSyntax<T> {}

impl<T> SealRefExpression for TypeAsSyntax<T> {}

impl<T> OptionalExpression for TypeAsSyntax<T> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_type_as_syntax {
    use super::TypeAsSyntax;
    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{RefExpression, StatementBuilder},
    };
    use sqlx::Type;

    impl<'a, T, S> RefExpression<'a, S> for TypeAsSyntax<T>
    where
        S: DatabaseStatementBuilder,
        T: Type<S>,
    {
        fn ref_expression<'q>(&'a self, ctx: &mut StatementBuilder<'q, S>) {
            S::type_as_syntax::<T>(&mut ctx.stmt);
        }
    }
}

#[derive(Clone)]
pub struct AliasedScopedColumn<T, C, A> {
    pub table: T,
    pub column: C,
    pub alias: A,
}

impl<T, C, A> SealExpression for AliasedScopedColumn<T, C, A> {}

impl<T, C, A> OptionalExpression for AliasedScopedColumn<T, C, A> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_aliased_scoped_column_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'q, T, C, A> Expression<'q, Sqlite> for super::AliasedScopedColumn<T, C, A>
    where
        C: Expression<'q, Sqlite>,
        T: Expression<'q, Sqlite>,
        A: Expression<'q, Sqlite>,
    {
        fn expression(self, arg: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            self.table.expression(arg);
            arg.syntax(".");
            self.column.expression(arg);
            arg.syntax(" AS ");
            self.alias.expression(arg);
        }
    }
}

#[derive(Clone)]
pub struct ScopedColumn<T, C> {
    pub table: T,
    pub col: C,
}

impl<T, C> SealExpression for ScopedColumn<T, C> {}

impl<T, C> OptionalExpression for ScopedColumn<T, C> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_scoped_column_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'q, T, C> Expression<'q, Sqlite> for super::ScopedColumn<T, C>
    where
        T: Expression<'q, Sqlite>,
        C: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            self.table.expression(ctx);
            ctx.syntax(".");
            self.col.expression(ctx);
        }
    }
}

#[derive(Clone)]
pub struct UpdatingColumn<C, T> {
    pub col: C,
    pub set: T,
}

impl<C, T> SealExpression for UpdatingColumn<C, Bind<T>> {}

impl<C, T> OptionalExpression for UpdatingColumn<C, Bind<T>> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_updating_column_bind_for_sqlite {
    use sqlx::{Encode, Sqlite, Type};

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'a, C, T> Expression<'a, Sqlite> for super::UpdatingColumn<C, super::Bind<T>>
    where
        T: Type<Sqlite> + Encode<'a, Sqlite> + 'a,
        C: Expression<'a, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'a, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            self.col.expression(ctx);
            ctx.syntax(" = ");
            super::Bind(self.set.0).expression(ctx);
        }
    }
}

impl<C, T> SealExpression for UpdatingColumn<C, Option<T>> {}

impl<C, T> OptionalExpression for UpdatingColumn<C, Option<T>> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_updating_column_option_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'a, C, T> Expression<'a, Sqlite> for super::UpdatingColumn<C, Option<T>>
    where
        T: Expression<'a, Sqlite>,
        C: Expression<'a, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'a, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            self.col.expression(ctx);
            ctx.syntax(" = ");
            match self.set {
                Some(value) => {
                    value.expression(ctx);
                }
                None => {
                    ctx.syntax("NULL");
                }
            }
        }
    }
}

impl<C, T> SealExpression for UpdatingColumn<C, Update<T>> {}

impl<C, T> OptionalExpression for UpdatingColumn<C, Update<T>> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_updating_column_update_for_sqlite {
    use sqlx::{Encode, Sqlite, Type};

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, RefExpression, StatementBuilder},
        update_mod::Update,
    };

    impl<'a, C, T> Expression<'a, Sqlite> for super::UpdatingColumn<C, Update<super::Bind<T>>>
    where
        T: Type<Sqlite> + Encode<'a, Sqlite> + 'a,
        C: for<'b> RefExpression<'b, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'a, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            match self.set {
                Update::Set(value) => {
                    self.col.ref_expression(ctx);
                    ctx.syntax(" = ");
                    super::Bind(value.0).expression(ctx);
                }
                Update::Keep => {}
            }
        }
    }
}

macro_rules! column_compare {
    ($name:ident, $field_name:ident, $op:literal) => {
        #[derive(Clone)]
        pub struct $name<Col, Val> {
            pub col: Col,
            pub $field_name: Val,
        }

        impl<Col, Val> SealExpression for $name<Col, Val> {}

        impl<Col, Val> OptionalExpression for $name<Col, Val> {
            fn is_expression_present(&self) -> bool {
                true
            }
        }

        paste::paste! {
            mod [<impl_ $name:snake _for_sqlite>] {
                use sqlx::Sqlite;

                use crate::{
                    extend_sqlx::DatabaseStatementBuilder,
                    sqlx_query_builder::{Expression, StatementBuilder},
                };

                impl<'q, Col, Val> Expression<'q, Sqlite> for super::$name<Col, Val>
                where
                    Col: Expression<'q, Sqlite>,
                    Val: Expression<'q, Sqlite>,
                {
                    fn expression(self, arg: &mut StatementBuilder<'q, Sqlite>)
                    where
                        Sqlite: DatabaseStatementBuilder,
                    {
                        self.col.expression(arg);
                        arg.syntax($op);
                        self.$field_name.expression(arg);
                    }
                }
            }
        }
    };
}

macro_rules! column_is {
    ($name:ident, $op:literal) => {
        #[derive(Clone)]
        pub struct $name<Col> {
            pub col: Col,
        }

        impl<Col> SealExpression for $name<Col> {}

        impl<Col> OptionalExpression for $name<Col> {
            fn is_expression_present(&self) -> bool {
                true
            }
        }

        paste::paste! {
            mod [<impl_ $name:snake _for_sqlite>] {
                use sqlx::Sqlite;

                use crate::{
                    extend_sqlx::DatabaseStatementBuilder,
                    sqlx_query_builder::{Expression, StatementBuilder},
                };

                impl<'q, Col> Expression<'q, Sqlite> for super::$name<Col>
                where
                    Col: Expression<'q, Sqlite>,
                {
                    fn expression(self, arg: &mut StatementBuilder<'q, Sqlite>)
                    where
                        Sqlite: DatabaseStatementBuilder,
                    {
                        self.col.expression(arg);
                        arg.syntax($op);
                    }
                }
            }
        }
    };
}

macro_rules! group_with {
    ($name:ident, $op:literal) => {
        #[derive(Clone)]
        pub struct $name<T>(pub T);

        impl<T: OptionalExpression> SealExpression for $name<T> {}

        impl<T: OptionalExpression> OptionalExpression for $name<T> {
            fn is_expression_present(&self) -> bool {
                self.0.is_expression_present()
            }
        }

        paste::paste! {
            mod [<impl_ $name:snake _for_sqlite>] {
                use sqlx::Sqlite;

                use crate::{
                    extend_sqlx::DatabaseStatementBuilder,
                    sqlx_query_builder::{Expression, Join, OptionalExpression, StatementBuilder},
                };

                impl<'q, T> Expression<'q, Sqlite> for super::$name<T>
                where
                    T: OptionalExpression,
                    Join<T>: Expression<'q, Sqlite>,
                {
                    fn expression(self, arg: &mut StatementBuilder<'q, Sqlite>)
                    where
                        Sqlite: DatabaseStatementBuilder,
                    {
                        if self.0.is_expression_present() {
                            Join {
                                start: "(",
                                separator: $op,
                                items: self.0,
                            }
                            .expression(arg);
                            arg.syntax(")");
                        }
                    }
                }
            }
        }
    };
}

column_compare!(ColumnEqual, eq, " = ");
column_compare!(ColumnNotEqual, ne, " != ");
column_compare!(ColumnGreaterThan, gt, " > ");
column_compare!(ColumnGreaterThanOrEqual, ge, " >= ");
column_compare!(ColumnLessThan, lt, " < ");
column_compare!(ColumnLessThanOrEqual, le, " <= ");
column_compare!(ColumnContains, like, " LIKE ");

column_is!(ColumnIsNotNull, " IS NOT NULL");
column_is!(ColumnIsNull, " IS NULL");

group_with!(ExpressionsWithAnd, " AND ");
group_with!(ExpressionsWithOr, " OR ");

#[derive(Clone)]
pub struct ColumnIn<Col, V> {
    pub col: Col,
    pub values: V,
}

impl<Col, V> SealExpression for ColumnIn<Col, V> {}

impl<Col, V: OptionalExpression> OptionalExpression for ColumnIn<Col, V> {
    fn is_expression_present(&self) -> bool {
        self.values.is_expression_present()
    }
}

mod impl_column_in_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, Join, OptionalExpression, StatementBuilder},
    };

    impl<'q, Col, V> Expression<'q, Sqlite> for super::ColumnIn<Col, V>
    where
        Col: Expression<'q, Sqlite>,
        Join<V>: Expression<'q, Sqlite>,
        V: OptionalExpression,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            if self.values.is_expression_present() {
                self.col.expression(ctx);
                Join {
                    start: " IN (",
                    separator: ", ",
                    items: self.values,
                }
                .expression(ctx);
                ctx.syntax(")");
            }
        }
    }
}

#[derive(Clone)]
pub struct ManyColumnsLargerOrEqual<Ids, Values> {
    pub ids: Ids,
    pub values: Values,
}

impl<Ids, Values> SealExpression for ManyColumnsLargerOrEqual<Ids, Values> {}

impl<Ids, Values> OptionalExpression for ManyColumnsLargerOrEqual<Ids, Values> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_many_columns_larger_or_equal_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, Join, StatementBuilder},
    };

    impl<'q, Ids, Values> Expression<'q, Sqlite> for super::ManyColumnsLargerOrEqual<Ids, Values>
    where
        Join<Ids>: Expression<'q, Sqlite>,
        Join<Values>: Expression<'q, Sqlite>,
    {
        fn expression(self, arg: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            arg.syntax("(");
            Join {
                start: "",
                separator: ",",
                items: self.ids,
            }
            .expression(arg);
            arg.syntax(")");
            arg.syntax(" >= ");
            arg.syntax("(");
            Join {
                start: "",
                separator: ",",
                items: self.values,
            }
            .expression(arg);
            arg.syntax(")");
        }
    }
}

pub struct ForeignKey<Table, Column, Ons> {
    pub references_table: Table,
    pub references_col: Column,
    pub ons: Ons,
}

impl<Table, Column, Ons> SealExpression for ForeignKey<Table, Column, Ons> {}

impl<Table, Column, Ons> OptionalExpression for ForeignKey<Table, Column, Ons> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod imp_foriegn_key_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, Join, StatementBuilder},
    };

    impl<'q, Table, Column, Ons> Expression<'q, Sqlite> for super::ForeignKey<Table, Column, Ons>
    where
        Join<Ons>: Expression<'q, Sqlite>,
        Table: Expression<'q, Sqlite>,
        Column: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("REFERENCES ");
            self.references_table.expression(ctx);
            ctx.syntax("(");
            self.references_col.expression(ctx);
            ctx.syntax(")");
            Join {
                start: " ",
                separator: ", ",
                items: self.ons,
            }
            .expression(ctx);
        }
    }
}

pub struct JoinExpression<ForeignTable, ForeignColumn, LocalTable, LocalColumn> {
    pub join_type: &'static str,
    pub foreign_table: ForeignTable,
    pub foreign_column: ForeignColumn,
    pub local_table: LocalTable,
    pub local_column: LocalColumn,
}

impl<ForeignTable, ForeignColumn, LocalTable, LocalColumn> SealExpression
    for JoinExpression<ForeignTable, ForeignColumn, LocalTable, LocalColumn>
{
}

impl<ForeignTable, ForeignColumn, LocalTable, LocalColumn> OptionalExpression
    for JoinExpression<ForeignTable, ForeignColumn, LocalTable, LocalColumn>
{
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_join_expression_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, RefExpression, StatementBuilder},
    };

    impl<'q, Ft, Fc, Lt, Lc> Expression<'q, Sqlite> for super::JoinExpression<Ft, Fc, Lt, Lc>
    where
        Ft: for<'a> RefExpression<'a, Sqlite>,
        Fc: Expression<'q, Sqlite>,
        Lt: Expression<'q, Sqlite>,
        Lc: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax(self.join_type);
            ctx.syntax(" ");
            self.foreign_table.ref_expression(ctx);
            ctx.syntax(" ON ");
            self.local_table.expression(ctx);
            ctx.syntax(".");
            self.local_column.expression(ctx);
            ctx.syntax(" = ");
            self.foreign_table.ref_expression(ctx);
            ctx.syntax(".");
            self.foreign_column.expression(ctx);
        }
    }
}

pub struct OnDeleteSetNull;

impl SealExpression for OnDeleteSetNull {}

impl OptionalExpression for OnDeleteSetNull {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod imp_on_delete_set_null_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'q> Expression<'q, Sqlite> for super::OnDeleteSetNull {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("ON DELETE SET NULL");
        }
    }
}

pub struct OnDeleteCascase;

impl SealExpression for OnDeleteCascase {}

impl OptionalExpression for OnDeleteCascase {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod imp_on_delete_cascase_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'q> Expression<'q, Sqlite> for super::OnDeleteCascase {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("ON DELETE CASCADE");
        }
    }
}

pub struct CompositePrimaryKey<Cols>(pub Cols);

impl<Cols> SealExpression for CompositePrimaryKey<Cols> {}

impl<Cols> OptionalExpression for CompositePrimaryKey<Cols> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_composite_primary_key_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'q, Cols> Expression<'q, Sqlite> for super::CompositePrimaryKey<Cols>
    where
        Cols: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("PRIMARY KEY (");
            self.0.expression(ctx);
            ctx.syntax(")");
        }
    }
}

pub struct DefaultExpression<T> {
    pub value: T,
}

impl<T> SealExpression for DefaultExpression<T> {}

impl<T> OptionalExpression for DefaultExpression<T> {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod impl_default_expression_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'q, T> Expression<'q, Sqlite> for super::DefaultExpression<T>
    where
        T: Expression<'q, Sqlite>,
    {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("DEFAULT ");
            self.value.expression(ctx);
        }
    }
}

pub struct CurrentTimestamp;

impl SealExpression for CurrentTimestamp {}

impl OptionalExpression for CurrentTimestamp {
    fn is_expression_present(&self) -> bool {
        true
    }
}

mod current_timestamp_for_sqlite {
    use sqlx::Sqlite;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{Expression, StatementBuilder},
    };

    impl<'q> Expression<'q, Sqlite> for super::CurrentTimestamp {
        fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
        where
            Sqlite: DatabaseStatementBuilder,
        {
            ctx.syntax("CURRENT_TIMESTAMP");
        }
    }
}

pub mod sub_query_expressions {
    use crate::sqlx_query_builder::{OptionalExpression, SealExpression};

    pub struct CurrentTable;
    pub struct ColumnMatchNew<C>(pub C);
    pub struct ColumnNewEqualsOld<C>(pub C);

    impl SealExpression for CurrentTable {}
    impl OptionalExpression for CurrentTable {
        fn is_expression_present(&self) -> bool {
            true
        }
    }
    impl<C> SealExpression for ColumnMatchNew<C> {}
    impl<C> OptionalExpression for ColumnMatchNew<C> {
        fn is_expression_present(&self) -> bool {
            true
        }
    }
    impl<C> SealExpression for ColumnNewEqualsOld<C> {}
    impl<C> OptionalExpression for ColumnNewEqualsOld<C> {
        fn is_expression_present(&self) -> bool {
            true
        }
    }

    mod impl_for_sqlite {
        use sqlx::Sqlite;

        use crate::{
            extend_sqlx::DatabaseStatementBuilder,
            sqlx_query_builder::{Expression, RefExpression, StatementBuilder},
        };

        impl<'q> Expression<'q, Sqlite> for super::CurrentTable {
            fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
            where
                Sqlite: DatabaseStatementBuilder,
            {
                ctx.syntax("{table}");
            }
        }

        impl<'q, C> Expression<'q, Sqlite> for super::ColumnMatchNew<C>
        where
            C: for<'b> RefExpression<'b, Sqlite>,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
            where
                Sqlite: DatabaseStatementBuilder,
            {
                self.0.ref_expression(ctx);
                ctx.syntax(" = NEW.");
                self.0.ref_expression(ctx);
            }
        }

        impl<'q, C> Expression<'q, Sqlite> for super::ColumnNewEqualsOld<C>
        where
            C: for<'b> RefExpression<'b, Sqlite>,
        {
            fn expression(self, ctx: &mut StatementBuilder<'q, Sqlite>)
            where
                Sqlite: DatabaseStatementBuilder,
            {
                ctx.syntax("NEW.");
                self.0.ref_expression(ctx);
                ctx.syntax(" = OLD.");
                self.0.ref_expression(ctx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;

    use crate::sqlx_query_builder::{
        Bind, Expression, Join, basic_expressions::ColumnEqual, combinators::Nest,
    };

    #[test]
    fn bind_and_column_equal() {
        let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(ColumnEqual {
            col: "column",
            eq: Bind(34),
        });
        assert_eq!(stmt, r#""column" = $1"#);
    }

    #[test]
    fn nested_joins_flatten() {
        let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(Join {
            start: "START ",
            separator: ", ",
            items: (
                Nest(("id", "email")),
                Nest(("name", "age")),
                Nest(("job", "job_description")),
            ),
        });
        assert_eq!(
            stmt,
            r#"START "id", "email", "name", "age", "job", "job_description""#
        );
    }
}
