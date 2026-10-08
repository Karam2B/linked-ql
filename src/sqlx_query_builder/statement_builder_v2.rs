use crate::extend_sqlx::DatabaseStatementBuilder;

pub struct StatementBuilder<'q, S>
where
    S: DatabaseStatementBuilder,
{
    pub(crate) stmt: String,
    pub(crate) info: S::StatementBuilderInfo,
    pub(crate) arg: S::Arguments<'q>,
}

impl<'q, S: DatabaseStatementBuilder> StatementBuilder<'q, S> {
    pub fn stmt(&self) -> &str {
        &self.stmt
    }

    /// push str that is known to not cause sql injection,
    ///
    /// the type for the syntax is `&'static str`, because
    /// it is less likely to be created by a network request
    /// (unless you maliciously leak a string like `String::from(from_network).leak()`)
    /// that is fine because on the backend you can only protect against bad code not malicious one
    pub fn syntax(&mut self, str: &'static str) {
        self.stmt.push_str(str);
    }
}

#[cfg(not(feature = "refactor"))]
/// refactoring todos:
/// - these methods should be removed in favor of having only 'Default' as
/// the public constructor
/// - any uses for this new method should be replaced by Expression::sql_statement
/// - any uses for this new_no_data method should be replaced by Expression::sql_statement_no_data
/// - on completing the refactoring with deletion istruction
///     delete this impl, it is kept only for reference during
///     refactoring
#[linked_sql_macros::skip]
impl<'q, S: DatabaseExt> StatementBuilder<'q, S> {
    pub fn new<Expr>(expr: Expr) -> Self
    where
        Expr: Expression<'q, S>,
    {
        let mut this = Self::default();

        expr.expression(&mut this);

        this
    }

    pub fn new_no_data<Expr>(expr: Expr) -> Option<String>
    where
        Expr: Expression<'q, S>,
    {
        let mut this = Self::default();

        expr.expression(&mut this);

        if this.count == 0 {
            Some(this.stmt)
        } else {
            None
        }
    }
}
