#[cfg(all(feature = "refactor", feature = "sqlite"))]
mod sqlite_statement_builder {
    use crate::extend_sqlx::{DatabaseExt, DatabaseStatementBuilder};
    use sqlx::{Arguments, Encode, Sqlite, Type, TypeInfo};

    impl DatabaseExt for Sqlite {}

    impl DatabaseStatementBuilder for Sqlite {
        type StatementBuilderInfo = usize;

        fn impl_bind<'q, V>(
            value: V,
            stmt: &mut String,
            info: &mut Self::StatementBuilderInfo,
            arg: &mut Self::Arguments<'q>,
        ) where
            V: Encode<'q, Self> + 'q + Type<Self>,
        {
            arg.add(value).expect("when does this ever fail?");
            *info += 1;
            stmt.push_str(&format!("${info}"));
        }

        fn is_buffer_empty(info: &Self::StatementBuilderInfo) -> bool {
            *info == 0
        }

        fn type_as_syntax<T: Type<Self>>(into: &mut String) {
            into.push_str(T::type_info().name());
        }

        fn sanitize_start(into: &mut String) {
            into.push('"');
        }

        fn sanitize_end(into: &mut String) {
            into.push('"');
        }

        fn sanitize_char(char: char, into: &mut String) {
            push_sanitized(char, into);
        }

        fn sanitize_segment(string: &str, into: &mut String) {
            for char in string.chars() {
                push_sanitized(char, into);
            }
        }
    }

    fn push_sanitized(char: char, into: &mut String) {
        match char {
            '"' | '\'' | '\\' => {
                into.push(char);
                into.push(char);
            }
            other => into.push(other),
        }
    }
}

mod connect_in_memory_for_sqlite {
    use crate::extend_sqlx::InMemoryConnection;
    use sqlx::Connection;
    use sqlx::{Pool, Sqlite, SqliteConnection};

    impl InMemoryConnection for Sqlite {
        fn in_memory_connection() -> impl Future<Output = Self::Connection> + Send {
            async {
                let conn = SqliteConnection::connect("file::memory:?cache=shared")
                    .await
                    .unwrap();
                conn
            }
        }

        fn in_memory_pool() -> impl Future<Output = sqlx::Pool<Self>> {
            async {
                let pool = Pool::<Sqlite>::connect("file::memory:?cache=shared")
                    .await
                    .unwrap();
                pool
            }
        }
    }
}

#[cfg(feature = "refactor")]
mod state_error_info_for_sqlite {
    use sqlx::Sqlite;

    use crate::extend_sqlx::database_error::{ErrorHandler, PropagateStateErrors, StateError};

    impl ErrorHandler<Sqlite> for PropagateStateErrors {
        #[track_caller]
        fn handle_sqlx_error(&self, err: sqlx::Error) -> Self::Error {
            match err {
                sqlx::Error::Database(ref e) => {
                    if e.is_foreign_key_violation() {
                        StateError::ForeignKeyViolation
                    } else if e.is_unique_violation() {
                        StateError::UniqueViolation
                    } else if let Some(e) = e.code() {
                        // notes:
                        // - look at https://www.sqlite.org/rescode.html
                        // - "1299" SQLITE_CONSTRAINT_NOTNULL is a schema error.
                        match e.as_ref() {
                            "2067" => StateError::UniqueViolation, // SQLITE_CONSTRAINT_UNIQUE
                            "1555" => StateError::UniqueViolation, // SQLITE_PRIMARYKEY
                            "2579" => StateError::UniqueViolation, // SQLITE_ROWID
                            "787" => StateError::ForeignKeyViolation, // SQLITE_CONSTRAINT_FOREIGNKEY
                            "1811" => StateError::Other,              // SQLITE_CONSTRAINT_TRIGGER
                            "275" => StateError::Other,               // SQLITE_CONSTRAINT_CHECK
                            _ => panic!("bug: {:?}", err),
                        }
                    } else {
                        panic!("bug: {:?}", err);
                    }
                }
                // schema
                sqlx::Error::Decode(_)
                | sqlx::Error::Encode(_)
                | sqlx::Error::RowNotFound
                | sqlx::Error::TypeNotFound { .. }
                | sqlx::Error::ColumnIndexOutOfBounds { .. }
                | sqlx::Error::ColumnNotFound(_)
                | sqlx::Error::ColumnDecode { .. } => {
                    panic!("bug: {:?}", err);
                }
                // operational
                sqlx::Error::Io(_)
                | sqlx::Error::Tls(_)
                | sqlx::Error::PoolTimedOut
                | sqlx::Error::PoolClosed
                | sqlx::Error::WorkerCrashed => {
                    panic!("{:?}", err);
                }
                // unsure, mostly operational
                _ => {
                    panic!("{:?}", err);
                }
            }
        }
    }
}
