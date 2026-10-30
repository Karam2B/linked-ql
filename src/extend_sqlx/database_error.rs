//! # DatabaseError
//! This module defines how sqlx::Error-s are propagated/paniced on.
//!
//! # Declaritive Infalliblility
//! Operation in this crate are expected to declare how they handle errors during
//! constructing their type, usually they clear out all errors an ORM is supposed to handle
//! and return errors that are the user's fault (called state errors).
//!
//! Look for (more information)[crate::operations_v2]
//!
//! # Dicecting sqlx::Error
//!
//! ## Schema Errors
//! A category of errors that a correct ORM implementation should
//! eliminate.
//!
//! - sqlx::Error::RowNotFound
//! - sqlx::Error::TypeNotFound
//! - sqlx::Error::ColumnIndexOutOfBounds
//! - sqlx::Error::ColumnNotFound
//! - sqlx::Error::ColumnDecode
//! - sqlx::Error::Encode
//! - sqlx::Error::Decode
//! - sqlx::Error::Database(e) && e.kind() is ErrorKind::NotNullViolation
//! - sqlx::Error::Database(e) && e.kind() is ErrorKind::CheckViolation
//!     will be assumed to be a state error, unless more information is available.
//!
//! ## State Errors
//! A category of errors that are caused by state of the database,
//! like unique key violation, foreign key violation, etc.
//!
//! - sqlx::Error::Database(e) && e.kind() is ErrorKind::ForeignKeyViolation
//! - sqlx::Error::Database(e) && e.kind() is ErrorKind::UniqueViolation
//! - sqlx::Error::Database(e) && e.kind() is ErrorKind::CheckViolation
//!     this is tricky, if there is a check constraint that depends on two columns, and you update only one
//!     that would be a case of state error, but if you update both columns, you should be able to anticipate
//!     the error, and this would be a case of schema error.
//! - sqlx::Error::Database(e) && e.kind() is ErrorKind::Other
//!     this is might be the way the db reports trigger violation (like sqlite error code 1811).
//!
//! This it the trickies type of errors, at some level, there
//! is no way to anticipate this error, but ultimately, this is
//! possible depending on the type of your application.
//!
//! ## Operational Errors
//! A category of errors that hardware limitation triggers,
//! like network timeout, disk I/O errors, etc.
//!
//! - sqlx::Error::Io
//! - sqlx::Error::Tls
//!
//! ## Unsure
//!
//! - sqlx::Error::Configuration
//! - sqlx::Error::InvalidArgument
//! - sqlx::Error::Database(e) && e.kind() is ErrorKind::Other
//! - sqlx::Error::AnyDriverError
//! - sqlx::Error::Protocol
//!     The documentation said the this might be caused by programming error in sqlx deriver code,
//!     in that case, sqlx should have paniced on this error and never propagate it to us.
//!     Other possibility is to be caused by problem in the connection,
//!     which make this error operational error.
//! - sqlx::Error::PoolTimedOut
//! - sqlx::Error::PoolClosed
//! - sqlx::Error::WorkerCrashed
//! - sqlx::Error::Migrate
//! - sqlx::Error::InvalidSavePointStatement
//! - sqlx::Error::BeginFailed
//! - non_execlusive
//!
//! These seems to be operational errors, but they are either poorly
//! documented, irrelavent to this crate, or not well tested yet.
//!
//! ## Level of Abstraction
//! If you are building an application where you are in control
//! of all its aspects, `Result` should always be anticipated or
//! panic on, and never propagated. This should be done in a way
//! user is informed of any possible error before it happens.
//!
//! That can be in the frontend as a UI error message, that prevent
//! the user from submitting an http request that would cause a
//! 4xx error
//!
//! Or that can be as an LSP error message, the prevent
//! the user from executing the code that would result in
//! sqlx::Error.
//!
//! I call this type of application, where you in controll
//! of all levels of abstraction, "closed system" application.
//!
//! That being said, it is challenging to create such an application.
//!
//! As far as ORM level of abstraction is concerned, schema and operational errors should
//! be paniced on, while state errors should be propagated to the user.
//!
//! Although at "closed system", state errors can
//! be anticipated. If you are aware that there is only one editor that
//! modify the database only through your applications, you can check
//! -- for example -- if a foreign key violation would occur or not.
//!
//! Consider this psudocode:
//!
//! ```
//! insert_todo(title: "first_todo") // LSP knows this is the first todo with id = 1
//! insert_todo(title: "second_todo") // LSP knows this is the second todo with id = 2
//!
//! add_todo_to_category(category: "work", todo_id: 1) // LSP pass this call
//! add_todo_to_category(category: "personal", todo_id: 3) // LSP knows there is no todo with id = 3, and will emit an error, before any sql is invoked.
//! ```
//!
//! The trait in this module is designed to target schema level of abstraction --
//! that is panic on schema and operational errors, and propagate state errors.
//!
//! # Testing
//! For now I have tested the behavior of sqlite, other databases behavior is
//! not guaranteed.

/*
    refactor todos:
    - this module replaces `operations::insert::AbortOperation`
    - this module repalces `operations::insert::DismissFailure`
    - this module replaces `operations::insert::ConstraintViolation`

*/

pub trait ErrorType {
    type Error;
}
pub trait ErrorHandler<S>: ErrorType {
    fn handle_sqlx_error(&self, err: sqlx::Error) -> Self::Error;
}

pub struct PropagateStateErrors;

/// minimal representation of how state errors are represented in the database.
/// obtained by specifying `PropagateStateErrors` in place where infalibility is expected.
pub enum StateError {
    /// Foreign key violation.
    ForeignKeyViolation,
    /// Unique key violation.
    UniqueViolation,
    /// A violation due to the value of other column in the same or other table.
    /// Example: check constraint that depends on two columns, trigger raised an error, etc.
    Other,
}

impl ErrorType for PropagateStateErrors {
    type Error = StateError;
}

pub struct DeclaritiveInfallible;

impl ErrorType for DeclaritiveInfallible {
    type Error = std::convert::Infallible;
}

impl<S> ErrorHandler<S> for DeclaritiveInfallible {
    #[track_caller]
    fn handle_sqlx_error(&self, err: sqlx::Error) -> Self::Error {
        panic!("{:?}", err);
    }
}

#[cfg(test)]
mod test_schema_errors_in_sqlite {
    use sqlx::{Row, Sqlite};

    use crate::extend_sqlx::InMemoryConnection;

    #[tokio::test]
    async fn slqx_column_decode() {
        let mut conn = Sqlite::in_memory_connection().await;

        let res = sqlx::query("SELECT $1")
            .bind("foo") // bind value is of type &str
            .fetch_one(&mut conn)
            .await
            .unwrap();

        // column with type &str is assumed to be i32.
        let err = res.try_get::<i32, _>(0).unwrap_err();

        assert!(matches!(err, sqlx::Error::ColumnDecode { .. }));
    }
}
