//! Decoding data from sqlx::Row types.
//!
//! This module differs from `sqlx::FromRow` in that:
//! - allows aliasing, which is important for links and operations
//! - seperate metas from data, which allows a more flexible system.
//! - focuses on errors that can be caused during decoding.
//!
use sqlx::{ColumnIndex, Decode, Row, Type};

pub mod swich_to_base_id {
    use sqlx::Row;

    use crate::from_row::{RowNumAliased, RowStrAliased};

    pub fn str_alias_to_base_id<'r, R: Row>(
        str_alias: RowStrAliased<'r, R>,
    ) -> RowStrAliased<'r, R> {
        RowStrAliased::new(str_alias.row, "i")
    }
    pub fn num_alias_to_base_id<'r, R: Row>(
        num_alias: RowNumAliased<'r, R>,
    ) -> RowNumAliased<'r, R> {
        RowNumAliased::new(num_alias.row, "i")
    }
}

pub struct RowStrAliased<'r, R: Row> {
    pub(crate) row: &'r R,
    pub(crate) alias: &'static str,
}

impl<'r, R: Row> Clone for RowStrAliased<'r, R> {
    fn clone(&self) -> Self {
        Self {
            row: self.row,
            alias: self.alias,
        }
    }
}

impl<'r, R: Row> RowStrAliased<'r, R> {
    pub fn new(row: &'r R, alias: &'static str) -> Self {
        Self { row, alias }
    }
    pub fn try_get<T>(&self, name: &str) -> Result<T, sqlx::Error>
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        Row::try_get(self.row, format!("{}{}", self.alias, name).as_str())
    }
    #[track_caller]
    pub fn get<T>(&self, name: &str) -> T
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        self.try_get(name).unwrap()
    }
}

pub struct RowNumAliased<'r, R: Row> {
    pub(crate) row: &'r R,
    pub(crate) str_alias: &'static str,
    // only Vec<T> and tuples, can initiate this with Some(usize)
    pub(crate) num_alias: Option<usize>,
}

impl<'r, R: Row> Clone for RowNumAliased<'r, R> {
    fn clone(&self) -> Self {
        Self {
            row: self.row,
            str_alias: self.str_alias,
            num_alias: self.num_alias,
        }
    }
}

impl<'r, R: Row> RowNumAliased<'r, R> {
    pub fn new(row: &'r R, name: &'static str) -> Self {
        Self {
            row,
            str_alias: name,
            num_alias: None,
        }
    }
    pub fn try_get<T>(&self, name: &str) -> Result<T, sqlx::Error>
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        Row::try_get(
            self.row,
            format!(
                "{}{}{}",
                self.str_alias,
                self.num_alias.map(|e| e.to_string()).unwrap_or_default(),
                name,
            )
            .as_str(),
        )
    }
    #[track_caller]
    pub fn get<T>(&self, name: &str) -> T
    where
        T: Type<R::Database> + Decode<'r, R::Database>,
        for<'q> &'q str: ColumnIndex<R>,
    {
        self.try_get(name).unwrap()
    }
}

#[derive(Debug)]
pub enum FromRowError {
    MismatchType,
    ColumnNotFound(String),
}

impl From<sqlx::Error> for FromRowError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::ColumnNotFound(name) => FromRowError::ColumnNotFound(name),
            _ => panic!(
                "{value:?}, either: 1. incorrect impl of FromRowAlias, 2. uncatchable error like database disconnection"
            ),
        }
    }
}

pub trait FromRowData {
    type RData;
}

pub trait FromRow<'r, R>: FromRowData {
    // used in operation that have no alias like returning clause
    fn from_row_no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError>;
}

pub trait FromRowAlias<'r, R>: FromRow<'r, R> {
    // used in operation that have local field belong to different links and collections
    fn from_row_str_alias(&self, row: RowStrAliased<'r, R>) -> Result<Self::RData, FromRowError>
    where
        R: Row;
    // used in links, where `Vec<T>` and tuples use an Option<usize>
    fn from_row_num_alias(&self, row: RowNumAliased<'r, R>) -> Result<Self::RData, FromRowError>
    where
        R: Row;
}

mod impl_from_row_for_tuples {
    use crate::from_row::FromRow;
    use crate::from_row::FromRowAlias;
    use crate::from_row::FromRowData;
    use crate::from_row::FromRowError;
    use crate::from_row::RowNumAliased;
    use crate::from_row::RowStrAliased;
    use sqlx::Row;

    macro_rules! impl_from_row_for_tuples {
        ($(($literal:literal, $type:ident))+) => {
            impl<$($type),*> FromRowData for ($($type,)*)
            where
                $($type: FromRowData,)*
            {
                type RData = ($(<$type as FromRowData>::RData,)*);
            }
            impl<'r, R, $($type,)*> FromRow<'r, R> for ($($type,)*)
            where
                R: Row,
                $($type: FromRow<'r, R>,)*
            {
                fn from_row_no_alias(&self, row: &'r R) -> Result<<Self as FromRowData>::RData, FromRowError> {
                    Ok(($(paste::paste!(self.$literal.from_row_no_alias(row)?),)*))
                }
            }
            impl<'r, R, $($type,)*> FromRowAlias<'r, R> for ($($type,)*)
            where
                R: Row,
                $($type: FromRowAlias<'r, R>,)*
            {
                fn from_row_str_alias(&self, row: RowStrAliased<'r, R>) -> Result<<Self as FromRowData>::RData, FromRowError> {
                    Ok(($(paste::paste!(self.$literal.from_row_str_alias(row.clone())?),)*))
                }

                fn from_row_num_alias(&self, row: RowNumAliased<'r, R>) -> Result<<Self as FromRowData>::RData, FromRowError> {
                    Ok(($(paste::paste!(self.$literal.from_row_num_alias(row.clone())?),)*))
                }
            }
        };
    }

    impl FromRowData for () {
        type RData = ();
    }

    impl<'r, R> FromRow<'r, R> for ()
    where
        R: Row,
    {
        fn from_row_no_alias(
            &self,
            _: &'r R,
        ) -> Result<<Self as FromRowData>::RData, FromRowError> {
            Ok(())
        }
    }

    impl<'r, R> FromRowAlias<'r, R> for ()
    where
        R: Row,
    {
        fn from_row_str_alias(
            &self,
            _: RowStrAliased<'r, R>,
        ) -> Result<<Self as FromRowData>::RData, FromRowError> {
            Ok(())
        }

        fn from_row_num_alias(
            &self,
            _: RowNumAliased<'r, R>,
        ) -> Result<<Self as FromRowData>::RData, FromRowError> {
            Ok(())
        }
    }

    impl_from_row_for_tuples!((0, T0));
    impl_from_row_for_tuples!((0, T0)(1, T1));
    impl_from_row_for_tuples!((0, T0)(1, T1)(2, T2));
    impl_from_row_for_tuples!((0, T0)(1, T1)(2, T2)(3, T3));
    impl_from_row_for_tuples!((0, T0)(1, T1)(2, T2)(3, T3)(4, T4));
    impl_from_row_for_tuples!((0, T0)(1, T1)(2, T2)(3, T3)(4, T4)(5, T5));
    impl_from_row_for_tuples!((0, T0)(1, T1)(2, T2)(3, T3)(4, T4)(5, T5)(6, T6));
    impl_from_row_for_tuples!((0, T0)(1, T1)(2, T2)(3, T3)(4, T4)(5, T5)(6, T6)(7, T7));
}

#[cfg(test)]
mod test {
    use sqlx::Sqlite;

    use crate::extend_sqlx::InMemoryConnection;
    use crate::{
        define_collection::collections_example::{Category, CategoryMeta},
        from_row::{FromRow, FromRowAlias, RowStrAliased},
    };

    #[tokio::test]
    async fn main() {
        let pool = Sqlite::in_memory_pool().await;

        let row = sqlx::query(
            "
        CREATE TABLE Category ( title TEXT );
        INSERT INTO Category (title) VALUES ('cat_1');
        SELECT title as cat_title, title  FROM Category;
    ",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let s = CategoryMeta
            .from_row_str_alias(RowStrAliased::new(&row, "cat_"))
            .unwrap();

        assert_eq!(
            s,
            Category {
                title: "cat_1".to_string(),
            },
        );

        let s = CategoryMeta.from_row_no_alias(&row).unwrap();

        assert_eq!(
            s,
            Category {
                title: "cat_1".to_string(),
            },
        );
    }
}
