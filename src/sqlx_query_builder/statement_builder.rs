use super::Expression;
use crate::database_extention::DatabaseExt;
use sqlx::{Encode, Type};

pub struct StatementBuilder<'q, S>
where
    S: DatabaseExt,
{
    pub(crate) stmt: String,
    count: usize,
    arg: S::Arguments<'q>,
}

// the only public constructor for StatementBuilder
impl<'q, S: DatabaseExt> Default for StatementBuilder<'q, S> {
    fn default() -> Self {
        StatementBuilder {
            stmt: String::new(),
            count: 0,
            arg: S::Arguments::default(),
        }
    }
}

impl<'q, S: DatabaseExt> StatementBuilder<'q, S> {
    pub fn stmt(&self) -> &str {
        &self.stmt
    }

    pub fn bind<V>(&mut self, value: V)
    where
        V: Encode<'q, S> + 'q + Type<S>,
    {
        use sqlx::Arguments;
        self.arg.add(value).expect("when does this ever fail?");
        self.count += 1;
        self.stmt.push_str(format!("${}", self.count).as_str());
    }

    pub fn sanitize(&mut self, display: &str) {
        S::sanitize_start(&mut self.stmt);
        S::sanitize(display, &mut self.stmt);
        S::sanitize_end(&mut self.stmt);
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

    pub fn type_as_syntax<T: Type<S>>(&mut self) {
        use sqlx::TypeInfo;
        self.stmt.push_str(T::type_info().name());
    }

    pub fn unwrap(self) -> (String, S::Arguments<'q>) {
        (self.stmt, self.arg)
    }
}

#[cfg(not(feature = "refactor"))]
/// refactoring todos:
/// - these methods should be removed in favor of having only 'Default' as
/// the public constructor
/// - any uses for this new method should be replaced by Expression::sql_statement
/// - any uses for this new_no_data method should be replaced by Expression::sql_statement_no_data
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
