use crate::connect_in_memory::ConnectInMemory;
use sqlx::{AnyPool, Pool};

impl ConnectInMemory for sqlx::Any {
    fn in_memory_connection() -> impl Future<Output = <Self as sqlx::Database>::Connection> + Send {
        async { todo!("impl ConnectInMemory::in_memory_connection for Any") }
    }

    fn in_memory_pool() -> impl Future<Output = Pool<Self>> {
        async {
            match AnyPool::connect("sqlite::memory:").await {
                Ok(pool) => pool,
                Err(error) => {
                    panic!("is error due to database not supporting in memory connections: {error}")
                }
            }
        }
    }
}
