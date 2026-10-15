//! Wraps sqlx queries.
//!
//! # Single vs multiple results
//! if the query returns S::Row or S::QueryResult, it is expected to use [Operation](crate::operations_v2::Operation).
//!
//! if the query return multiple rows, it is expected to use [StreamedOperation](crate::operations_v2::StreamedOperation).
//!
//! the output of the operation is specified by a non-generic trait [OperationOutput](crate::operations_v2::OperationOutput).
//!
//! All implementation of [StreamedOperation](crate::operations_v2::StreamedOperation) can be converted to [Operation](crate::operations_v2::Operation) as follows:
//! - streamed_ops::OperationVec<T>: returns a vector of the output of the operation
//! - streamed_ops::OperationHashed<T>: returns a hashmap if the output is (Key, Value)
//! - streamed_ops::OneRecordOperation<T, PanicOnFailure>: returns only one record. panics if got multiple
//! - streamed_ops::OneRecordOperation<T, ResultOnFailure>: returns only one record. returns an error if got multiple
//!
//! # Connection
//! Operations are expected to run as a transactions.
//!
//! Operation methods take `&mut S::Connection` you can obtain it from `sqlx::Connection::connect`,
//! if you have [Pool](sqlx::Pool) you can obtain it from `sqlx::Pool::begin`
//!
//! # Fallibility
//! Check the [fallibility](crate::operations_v2::fallibility) module for more details.
//!

use futures::Stream;
use sqlx::Database;

pub mod insert_one;
pub mod fallibility {
    //! # Infalibility
    //! All operations are supposed to be infalible, if they can fail,
    //! failure should be handled declaratively:
    //!
    //! ```ignore
    //! pub struct InsertOne<Infalibility> {
    //!     ... some code ...
    //! }
    //!
    //! impl OperationOutput for InsertOne<PanicOnFailure> {
    //!     type Output = OutputType;
    //! }
    //!
    //! impl OperationOutput for InsertOne<ResultOnFailure> {
    //!     type Output = Result<OutputType, ()>;
    //! }
    //!
    //! ```
    //!
    //! There are two categories of failures:
    //!     - what this crate expects to clear
    //!         - syntax errors
    //!         - not null constraint violation
    //!         - check constraint violation
    //!     - errors depends on the state of the database, like
    //!         - foreign key violation
    //!         - unique constraint violation
    //!         - check constraint violation that depends on values of columns

    //!
    //! Any error of the first category is considered to be a bug in linked_ql.
    //! Otherwise, the user has to be given a chance to handle the error, using any of
    //! [Fallibility](crate::operations_v2::fallibility::Fallibility)
    //!
    //! Some constraints can be better handled by creating new type,
    //! consider an integer that has to be more than 10, you can have
    //! ```ignore
    //!
    //! pub struct ShoppingEntity {
    //!     pub name: String,
    //!     pub price: MoreThanTen,
    //! }
    //!
    //! sqlx_new_type! {
    //!     pub struct MoreThanTen(i32);
    //! }
    //!
    //! impl MoreThanTen {
    //!     pub fn new(value: i32) -> Option<Self> {
    //!         if value > 10 {
    //!             Some(Self(value))
    //!         } else {
    //!             None
    //!         }
    //!     }
    //! }
    //! ```
    //!
    //! In this fation it's the user's responsibility to clear the error before constructing
    //! an entity.
    //!
    //! ------
    //! In addition to the two categories above, there are errors that pose design challenges.
    //!
    //! On theory all links are optional, and id is auto generated. but consider these cases:
    //!
    //! 1. missing non-auto-generated id
    //!
    //!
    //! 2. missing required link
    //!

    pub struct PanicOnFailure;
    pub struct ResultOnFailure;
}

/// refactor todo:
/// - I have some reimports, replace them by original code from crate::operations::*
/// - replacing the original item's documentation by the documentation of the reimport
/// - if the orignial item impl serde trait, remove that implementation

/// See the [module level documentation](crate::operations_v2) for more details.
pub use crate::operations::OperationOutput;

/// See the [module level documentation](crate::operations_v2) for more details.
pub use crate::operations::Operation;

/// See the (module level documentation)[] for more details.
pub trait StreamedOperation<S>: OperationOutput<Output: Send> + Send {
    fn stream_operation(self, conn: &mut S::Connection) -> impl Stream<Item = Self::Output> + Send
    where
        Self: Sized,
        S: Database;
}

pub use crate::operations::CollectionOutput;
pub use crate::operations::IdOutput;
pub use crate::operations::LinkedOutput;
pub use crate::operations::ManyLinkOutput;
pub use crate::operations::ManyOutput;

pub mod streamed_ops {
    use std::{collections::HashMap, hash::Hash, pin::pin};

    use futures::StreamExt;

    use crate::operations_v2::{
        Operation, OperationOutput, StreamedOperation,
        fallibility::{PanicOnFailure, ResultOnFailure},
    };

    /// Streamed operation collected as a vector
    pub struct OperationVec<T>(pub T);

    impl<T> OperationOutput for OperationVec<T>
    where
        T: OperationOutput,
    {
        type Output = Vec<T::Output>;
    }

    impl<S, T> Operation<S> for OperationVec<T>
    where
        T: OperationOutput,
        T: StreamedOperation<S>,
    {
        fn exec_operation(
            self,
            conn: &mut <S>::Connection,
        ) -> impl Future<Output = Self::Output> + Send
        where
            Self: Sized,
            S: sqlx::Database,
        {
            async move {
                let mut stream = pin!(self.0.stream_operation(conn));
                let mut output = Vec::new();
                while let Some(item) = stream.next().await {
                    output.push(item);
                }
                output
            }
        }
    }

    /// Streamed operation collected as a hashmap, where the output is (Key, Value)
    pub struct OperationHashed<T>(pub T);

    impl<T, Key, Value> OperationOutput for OperationHashed<T>
    where
        T: OperationOutput<Output = (Key, Value)>,
    {
        type Output = HashMap<Key, Vec<Value>>;
    }

    impl<S, T, Key, Value> Operation<S> for OperationHashed<T>
    where
        T: OperationOutput<Output = (Key, Value)>,
        T: StreamedOperation<S>,
        Value: Send,
        Key: Send + Eq + Hash,
    {
        fn exec_operation(
            self,
            conn: &mut <S>::Connection,
        ) -> impl Future<Output = Self::Output> + Send
        where
            Self: Sized,
            S: sqlx::Database,
        {
            async move {
                let mut stream = pin!(self.0.stream_operation(conn));
                let mut output = HashMap::new();
                while let Some((fi, ti)) = stream.next().await {
                    output.entry(fi).or_insert_with(|| Vec::new()).push(ti)
                }
                output
            }
        }
    }

    /// collect one record of a streamed operation, for infalibility you have two options:
    /// - PanicOnFailure: panic if got multiple records
    /// - ResultOnFailure: return an error if got multiple records
    pub struct OneRecordOperation<T, Infalibility> {
        pub operation: T,
        pub infalibility: Infalibility,
    }

    impl<T> OperationOutput for OneRecordOperation<T, PanicOnFailure>
    where
        T: OperationOutput,
    {
        type Output = Option<T::Output>;
    }

    impl<S, T> Operation<S> for OneRecordOperation<T, PanicOnFailure>
    where
        T: OperationOutput,
        T: StreamedOperation<S>,
        S: sqlx::Database,
    {
        fn exec_operation(
            self,
            conn: &mut <S>::Connection,
        ) -> impl Future<Output = Self::Output> + Send {
            async move {
                let mut stream = pin!(self.operation.stream_operation(conn));

                let output = stream.next().await;

                if stream.next().await.is_some() {
                    panic!("expected only one record, but got multiple");
                }

                output
            }
        }
    }

    impl<T> OperationOutput for OneRecordOperation<T, ResultOnFailure>
    where
        T: OperationOutput,
    {
        type Output = Result<Option<T::Output>, ()>;
    }

    impl<S, T> Operation<S> for OneRecordOperation<T, ResultOnFailure>
    where
        T: OperationOutput,
        T: StreamedOperation<S>,
        S: sqlx::Database,
    {
        fn exec_operation(
            self,
            conn: &mut <S>::Connection,
        ) -> impl Future<Output = Self::Output> + Send {
            async move {
                let mut stream = pin!(self.operation.stream_operation(conn));

                let output = stream.next().await;

                if stream.next().await.is_some() {
                    return Err(());
                }

                Ok(output)
            }
        }
    }
}

pub use crate::operations::boxed_operation;

pub mod execute_expression {
    use super::{Operation, OperationOutput};
    use crate::execute::Executable;
    use crate::extend_sqlx::DatabaseStatementBuilder;
    use crate::fix_executor::ExecutorTrait;
    use crate::sqlx_query_builder::Expression;
    use sqlx::Database;

    /// Convert an expression to an operation
    pub struct ExpressionAsOperation<E>(pub E);

    impl<E: Send> OperationOutput for ExpressionAsOperation<E> {
        type Output = ();
    }

    impl<S, E> Operation<S> for ExpressionAsOperation<E>
    where
        E: for<'q> Expression<'q, S>,
        E: Send,
        S: DatabaseStatementBuilder,
        S: ExecutorTrait,
    {
        async fn exec_operation(self, pool: &mut S::Connection) -> Self::Output
        where
            S: Database,
        {
            let (stmt, arg) = self.0.sql_statement();

            S::execute(
                pool,
                Executable {
                    string: stmt.as_str(),
                    arguments: arg,
                },
            )
            .await
            .unwrap();
        }
    }
}
