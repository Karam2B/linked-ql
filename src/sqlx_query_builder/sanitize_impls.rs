use crate::sqlx_query_builder::{
    Expression, RefExpression, SealExpression, SealRefExpression, StatementBuilder,
};
use std::sync::Arc;

impl SealExpression for String {}

impl SealRefExpression for String {}

impl<S> RefExpression<'_, S> for String
where
    S: DatabaseExt,
{
    fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
        ctx.sanitize_start();
        ctx.sanitize(self.as_str());
        ctx.sanitize_end();
    }
}

impl SealExpression for Arc<str> {}

impl SealRefExpression for Arc<str> {}

impl<S> RefExpression<'_, S> for Arc<str>
where
    S: DatabaseExt,
{
    fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
        ctx.sanitize_start();
        ctx.sanitize(self.as_str());
        ctx.sanitize_end();
    }
}

impl SealExpression for &'_ str {}

impl SealRefExpression for &'_ str {}

impl<S> RefExpression<'_, S> for &'_ str
where
    S: DatabaseExt,
{
    fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
        ctx.sanitize_start();
        ctx.sanitize(self);
        ctx.sanitize_end();
    }
}

pub struct Sanitize<T>(pub T);

mod tuple_element_supported {
    pub trait ToMutString {
        fn to_string(&self, str: &mut String);
    }

    impl<T: AsRef<str>> ToMutString for T {
        fn to_string(&self, str: &mut String) {
            str.push_str(self.as_ref());
        }
    }

    impl ToMutString for usize {
        fn to_string(&self, str: &mut String) {
            const MAX_DEC_N: usize = usize::MAX.ilog10() as usize + 1;
            let mut buf = [0u8; MAX_DEC_N];
            let mut current_index = 0;
            let mut member = *self;
            if member == 0 {
                stmt.push('0');
                return;
            }
            while member > 0 {
                let least_significant_digit = member % 10;
                buf[current_index] = least_significant_digit as u8 + b'0';
                current_index += 1;
                member /= 10;
            }
            let mut buf = buf[..current_index].into_iter();
            while let Some(digit) = buf.next_back() {
                stmt.push(*digit as char);
            }
        }
    }
}

mod impl_sanitize_for_std_types {
    impl RefExpression<'a, S> for Sanitize<Vec<T>>
    where
        S: DatabaseStatementBuilder,
        T: ToMutString,
    {
        fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize_start();
            self.0.iter().for_each(|item| item.to_string(&mut ctx.stmt));
            ctx.sanitize_end();
        }
    }
}

mod impl_sanitize_for_tuples {
    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        tuple_trait::{RefTuple, TupleSpecRef},
    };

    pub struct TupleSpecifier<'a, 'q, S>(&'a mut StatementBuilder<'q, S>, PhantomData<S>);

    impl RefExpression<'a, S> for Sanitize<T>
    where
        S: DatabaseStatementBuilder,
        T: RefTuple<TupleSpecifier>,
    {
        fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
            ctx.sanitize_start();
            self.0.ref_tuple_mut_spec(TupleSpecifier(ctx, PhantomData));
            ctx.sanitize_end();
        }
    }
}
