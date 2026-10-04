#[cfg(not(feature = "in_dev_op2"))]
use crate::operations::operations_expressions_crossover::TableExpressions;

#[cfg(not(feature = "in_dev_op2"))]
pub type ConventionalForeignKeyName<Key, Table> = (
    &'static str,
    (<Table as TableExpressions>::SnakeCase,),
    (Key,),
);

#[cfg(not(feature = "in_dev_op2"))]
pub fn conventional_foreign_key_name<Key, Table>(
    key: Key,
    table: &Table,
) -> ConventionalForeignKeyName<Key, Table>
where
    Key: AsRef<str>,
    Table: TableExpressions,
{
    ("fk_", (table.table_name_snake_case(),), (key,))
}

#[cfg(not(feature = "in_dev_op2"))]
pub type ConventionalJunctionTableName<Key, Table1, Table2> = (
    &'static str,
    (<Table1 as TableExpressions>::SnakeCase,),
    &'static str,
    (<Table2 as TableExpressions>::SnakeCase,),
    (Key,),
);

#[cfg(not(feature = "in_dev_op2"))]
pub fn conventional_junction_table_name<Key, Table1, Table2>(
    key: Key,
    table1: Table1,
    table2: Table2,
) -> ConventionalJunctionTableName<Key, Table1, Table2>
where
    Key: AsRef<str>,
    Table1: TableExpressions,
    Table2: TableExpressions + 'static,
{
    (
        "ct_",
        (table1.table_name_snake_case(),),
        "_",
        (table2.table_name_snake_case(),),
        (key,),
    )
}
