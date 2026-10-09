#![cfg(feature = "refactor")]

use linked_sql::sqlx_query_builder::{
    Bind, Expression, Join, OptionalExpression, RefExpression, Sanitize,
    basic_expressions::{ColumnEqual, TypeAsSyntax},
    combinators::{Nest, Prefixed},
    trait_objects::box_expression,
};
use sqlx::Sqlite;
use std::marker::PhantomData;

#[test]
fn bind_column_equal_and_nested_join() {
    let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(ColumnEqual {
        col: "column",
        eq: Bind(34),
    });
    assert_eq!(stmt, r#""column" = $1"#);

    let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(Join {
        start: "SELECT ",
        separator: ", ",
        items: (
            Nest(("column_1", "column_2")),
            Nest(("column_3", "column_4")),
        ),
    });
    assert_eq!(
        stmt,
        r#"SELECT "column_1", "column_2", "column_3", "column_4""#
    );
}

#[test]
fn empty_pieces_write_no_sql() {
    let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(Join {
        start: "WHERE ",
        separator: " AND ",
        items: Some("id"),
    });
    assert_eq!(stmt, r#"WHERE "id""#);

    let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(Join {
        start: "WHERE ",
        separator: " AND ",
        items: Option::<&str>::None,
    });
    assert_eq!(stmt, "");

    let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(Prefixed {
        prefix: " LIMIT ",
        inner: Option::<Bind<i32>>::None,
    });
    assert_eq!(stmt, "");
}

#[test]
fn sanitize_vec_and_tuple() {
    let stmt = RefExpression::<'_, Sqlite>::ref_sql_statement(&Sanitize(vec![1usize, 2, 3]));
    assert_eq!(stmt, "\"123\"");

    let hello = String::from("hello_");
    let world = String::from("world_");
    let stmt = RefExpression::<'_, Sqlite>::ref_sql_statement(&Sanitize((
        (hello.as_str(),),
        Some(world.as_str()),
        32usize,
    )));
    assert_eq!(stmt, "\"hello_world_32\"");
}

#[test]
fn type_name_is_written_without_binding() {
    let stmt = RefExpression::<'_, Sqlite>::ref_sql_statement(&TypeAsSyntax::<i32>(PhantomData));
    assert_eq!(stmt, "INTEGER");
}

#[test]
fn boxed_empty_expression_is_absent() {
    let boxed = box_expression::<Sqlite, Option<&str>>(None, ", ");
    assert!(!boxed.is_expression_present());

    let (stmt, _args) = Expression::<'_, Sqlite>::sql_statement(Join {
        start: "SELECT ",
        separator: ", ",
        items: (
            boxed,
            box_expression::<Sqlite, (&str, &str)>(("id", "name"), ", "),
        ),
    });
    assert_eq!(stmt, r#"SELECT "id", "name""#);
}
