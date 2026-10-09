use crate::{
    extend_sqlx::DatabaseStatementBuilder,
    sqlx_query_builder::{Expression, SealExpression, StatementBuilder},
};
/// refactoring todos:
/// - Remove the boxed_is_op_own method
/// - Remove box_expression function

pub trait BoxedExpression<S: DatabaseStatementBuilder>: Send {
    fn boxed_expression<'q>(self: Box<Self>, ctx: &mut StatementBuilder<'q, S>);
    #[cfg(not(feature = "refactor"))]
    fn boxed_is_op_own(&self) -> bool;
}

impl<E, S> BoxedExpression<S> for E
where
    S: DatabaseStatementBuilder,
    E: for<'e> Expression<'e, S> + Send,
{
    fn boxed_expression<'q>(self: Box<Self>, ctx: &mut StatementBuilder<'q, S>) {
        Expression::expression(*self, ctx);
    }

    #[cfg(not(feature = "refactor"))]
    fn boxed_is_op_own(&self) -> bool {
        // E: OptionalExpression,
        self.is_expression_present()
    }
}

impl<S: DatabaseStatementBuilder> BoxedExpression<S> for () {
    fn boxed_expression<'q>(self: Box<Self>, _: &mut StatementBuilder<'q, S>) {}

    #[cfg(not(feature = "refactor"))]
    fn boxed_is_op_own(&self) -> bool {
        false
    }
}

impl<S> SealExpression for Box<dyn BoxedExpression<S> + Send> where S: DatabaseStatementBuilder {}

#[cfg(not(feature = "refactor"))]
impl<S> crate::sqlx_query_builder::OptionalExpression for Box<dyn BoxedExpression<S> + Send>
where
    S: DatabaseStatementBuilder,
{
    fn is_expression_present(&self) -> bool {
        BoxedExpression::boxed_is_op_own(self.as_ref())
    }
}

impl<'q, S: DatabaseStatementBuilder> Expression<'q, S> for Box<dyn BoxedExpression<S> + Send> {
    fn expression(self, ctx: &mut StatementBuilder<'q, S>) {
        BoxedExpression::boxed_expression(self, ctx);
    }
}

#[cfg(not(feature = "refactor"))]
pub fn box_expression<S, T>(items: T, separator: &'static str) -> Box<dyn BoxedExpression<S> + Send>
where
    S: DatabaseStatementBuilder,
    T: OptionalExpression + Send + 'static,
    Join<T>: for<'e> Expression<'e, S> + OptionalExpression + Send,
{
    if !items.is_expression_present() {
        Box::new(())
    } else {
        Box::new(Join {
            start: "",
            separator,
            items,
        })
    }
}
