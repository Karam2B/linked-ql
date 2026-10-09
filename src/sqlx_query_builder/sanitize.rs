/// Sanitize multiple items in a single expression.
///
/// Supported for:
/// - Vec<T>
/// - Option<T>
/// - tuples where each member is T
///
/// where T is either:
/// - (&T,) where T: AsRef<str>
/// - Option<&T> where T: AsRef<str>
/// - usize
pub struct Sanitize<T>(pub T);

mod std_impls {
    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{RefExpression, SealExpression, SealRefExpression, StatementBuilder},
    };
    use std::sync::Arc;

    impl SealExpression for String {}

    impl SealRefExpression for String {}

    impl<S> RefExpression<'_, S> for String
    where
        S: DatabaseStatementBuilder,
    {
        fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
            S::sanitize_start(&mut ctx.stmt);
            S::sanitize_segment(self.as_str(), &mut ctx.stmt);
            S::sanitize_end(&mut ctx.stmt);
        }
    }

    impl SealExpression for Arc<str> {}

    impl SealRefExpression for Arc<str> {}

    impl<S> RefExpression<'_, S> for Arc<str>
    where
        S: DatabaseStatementBuilder,
    {
        fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
            S::sanitize_start(&mut ctx.stmt);
            S::sanitize_segment(self.as_ref(), &mut ctx.stmt);
            S::sanitize_end(&mut ctx.stmt);
        }
    }

    impl SealExpression for &'_ str {}

    impl SealRefExpression for &'_ str {}

    impl<S> RefExpression<'_, S> for &'_ str
    where
        S: DatabaseStatementBuilder,
    {
        fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
            S::sanitize_start(&mut ctx.stmt);
            S::sanitize_segment(self, &mut ctx.stmt);
            S::sanitize_end(&mut ctx.stmt);
        }
    }
}

mod patched_to_string_trait {
    use crate::{extend_sqlx::DatabaseStatementBuilder, sqlx_query_builder::StatementBuilder};

    /// merging types into a string, keeping it sanitized
    pub trait ToMutString<S: DatabaseStatementBuilder> {
        fn to_mut_string(&self, stmt: &mut StatementBuilder<'_, S>);
    }

    impl<T, S: DatabaseStatementBuilder> ToMutString<S> for (&'_ T,)
    where
        T: AsRef<str> + ?Sized,
    {
        fn to_mut_string(&self, stmt: &mut StatementBuilder<'_, S>) {
            S::sanitize_segment(self.0.as_ref(), &mut stmt.stmt);
        }
    }

    impl<T, S: DatabaseStatementBuilder> ToMutString<S> for Option<&'_ T>
    where
        T: AsRef<str> + ?Sized,
    {
        fn to_mut_string(&self, stmt: &mut StatementBuilder<'_, S>) {
            if let Some(item) = self {
                S::sanitize_segment(item.as_ref(), &mut stmt.stmt);
            }
        }
    }

    impl<S: DatabaseStatementBuilder> ToMutString<S> for usize {
        fn to_mut_string(&self, stmt: &mut StatementBuilder<'_, S>) {
            const MAX_DEC_N: usize = usize::MAX.ilog10() as usize + 1;
            let mut buf = [0u8; MAX_DEC_N];
            let mut current_index = 0;
            let mut member = *self;
            if member == 0 {
                S::sanitize_char('0', &mut stmt.stmt);
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
                S::sanitize_char(*digit as char, &mut stmt.stmt);
            }
        }
    }
}

mod impl_sanitize_for_std_types {
    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{
            RefExpression, SealExpression, SealRefExpression, StatementBuilder, sanitize::Sanitize,
        },
    };

    impl<T> SealExpression for Sanitize<T> {}

    impl<T> SealRefExpression for Sanitize<T> {}

    impl<'a, S, T> RefExpression<'a, S> for Sanitize<Vec<T>>
    where
        S: DatabaseStatementBuilder,
        T: super::patched_to_string_trait::ToMutString<S>,
    {
        fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
            S::sanitize_start(&mut ctx.stmt);
            self.0.iter().for_each(|item| item.to_mut_string(ctx));
            S::sanitize_end(&mut ctx.stmt);
        }
    }

    impl<'a, S, T> RefExpression<'a, S> for Sanitize<Option<T>>
    where
        S: DatabaseStatementBuilder,
        T: super::patched_to_string_trait::ToMutString<S>,
    {
        fn ref_expression<'q>(&self, ctx: &mut StatementBuilder<'q, S>) {
            if let Some(item) = &self.0 {
                S::sanitize_start(&mut ctx.stmt);
                item.to_mut_string(ctx);
                S::sanitize_end(&mut ctx.stmt);
            }
        }
    }
}

mod impl_sanitize_for_tuples {
    use std::marker::PhantomData;

    use crate::{
        extend_sqlx::DatabaseStatementBuilder,
        sqlx_query_builder::{RefExpression, Sanitize, StatementBuilder},
        tuple_trait::{RefTuple, TupleSpecRef},
    };

    pub struct TupleSpecifier<'a, 'q, S: DatabaseStatementBuilder>(
        &'a mut StatementBuilder<'q, S>,
        PhantomData<S>,
    );

    /// AsRefStr is wraped in tuple to avoid upstream crate from implementing AsRef<str>
    impl<'a, 'q, S, T> TupleSpecRef<T> for TupleSpecifier<'a, 'q, S>
    where
        T: super::patched_to_string_trait::ToMutString<S>,
        S: DatabaseStatementBuilder,
    {
        type Output = ();

        fn on_each<const LAST_INDEX: usize, const INDEX: usize>(
            &mut self,
            member: &T,
        ) -> Self::Output {
            member.to_mut_string(&mut self.0);
        }
    }

    impl<'a, S, T> RefExpression<'a, S> for Sanitize<T>
    where
        S: DatabaseStatementBuilder,
        T: for<'m, 'q> RefTuple<TupleSpecifier<'m, 'q, S>>,
    {
        fn ref_expression<'q>(&'a self, ctx: &mut StatementBuilder<'q, S>) {
            S::sanitize_start(&mut ctx.stmt);
            self.0.ref_tuple_mut_spec(TupleSpecifier(ctx, PhantomData));
            S::sanitize_end(&mut ctx.stmt);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sqlx_query_builder::RefExpression;
    use crate::sqlx_query_builder::sanitize::Sanitize;

    #[test]
    fn test_sanitize_vec() {
        let stmt =
            RefExpression::<'_, sqlx::Sqlite>::ref_sql_statement(&Sanitize(vec![1usize, 2, 3]));
        assert_eq!(stmt, "\"123\"");

        let hello = String::from("hello_");
        let world = String::from("world_");
        let stmt = RefExpression::<'_, sqlx::Sqlite>::ref_sql_statement(&Sanitize((
            (hello.as_str(),),
            Some(world.as_str()),
            32usize,
        )));
        assert_eq!(stmt, "\"hello_world_32\"");
    }
}
