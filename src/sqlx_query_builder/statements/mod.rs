#[cfg(feature = "refactor")]
#[path = "add_column_statement_v2.rs"]
pub mod add_column_statement;
#[cfg(not(feature = "refactor"))]
pub mod add_column_statement;

#[cfg(feature = "refactor")]
#[path = "create_table_statement_v2.rs"]
pub mod create_table_statement;
#[cfg(not(feature = "refactor"))]
pub mod create_table_statement;

#[cfg(feature = "refactor")]
#[path = "create_trigger_v2.rs"]
pub mod create_trigger;
#[cfg(not(feature = "refactor"))]
pub mod create_trigger;

#[cfg(feature = "refactor")]
#[path = "delete_statement_v2.rs"]
pub mod delete_statement;
#[cfg(not(feature = "refactor"))]
pub mod delete_statement;

#[cfg(feature = "refactor")]
#[path = "insert_statement_v2.rs"]
pub mod insert_statement;
#[cfg(not(feature = "refactor"))]
pub mod insert_statement;

#[cfg(feature = "refactor")]
#[path = "select_statement_v2.rs"]
pub mod select_statement;
#[cfg(not(feature = "refactor"))]
pub mod select_statement;

#[cfg(feature = "refactor")]
#[path = "update_statement_v2.rs"]
pub mod update_statement;
#[cfg(not(feature = "refactor"))]
pub mod update_statement;

pub trait Inverse {
    type InverseStatement;
    fn inverse(&self) -> Self::InverseStatement;
}
