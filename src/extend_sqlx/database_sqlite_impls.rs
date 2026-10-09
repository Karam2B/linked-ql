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
