use crate::{
    database::Database,
    executor::{ExecuteError, QueryResult, execute_sql_statement},
    parser::{ParseSqlErrors, parse_sql},
    tokenizer::{TokenizeError, tokenize},
};

pub(crate) enum QueryError {
    Tokenize(TokenizeError),
    Parse(ParseSqlErrors),
    Execute(ExecuteError),
}

pub(crate) fn run_query<'db>(
    database: &'db mut Database,
    sql: &str,
) -> Result<QueryResult<'db>, QueryError> {
    let tokens = tokenize(sql).map_err(QueryError::Tokenize)?;
    let statement = parse_sql(&tokens).map_err(QueryError::Parse)?;
    execute_sql_statement(database, &statement).map_err(QueryError::Execute)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        column::{Column, ColumnDataType},
        database::Database,
        executor::{ExecuteError, QueryResult},
        parser::ParseSqlErrors,
        query::{QueryError, run_query},
        row::{Row, RowValue},
        table::Table,
        tokenizer::TokenizeError,
    };

    #[test]
    fn query_e2e_from_sql_string_to_result_data() {
        let mut tables = HashMap::new();
        tables.insert(
            String::from("users"),
            Table {
                name: String::from("users"),
                columns: vec![
                    Column {
                        name: String::from("id"),
                        data_type: ColumnDataType::Number,
                    },
                    Column {
                        name: String::from("name"),
                        data_type: ColumnDataType::Text,
                    },
                ],
                rows: vec![
                    Row {
                        values: vec![RowValue::Number(1), RowValue::Text(String::from("test"))],
                    },
                    Row {
                        values: vec![RowValue::Number(2), RowValue::Text(String::from("test_2"))],
                    },
                ],
            },
        );

        let mut database = Database {
            name: String::from("test_database"),
            tables,
        };

        let sql_sentence = "SELECT * FROM users;";

        let result = run_query(&mut database, sql_sentence);

        let table = match result {
            Ok(QueryResult::Selected(table)) => table,
            _ => panic!("expected the users table"),
        };

        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.columns[0].name, "id");
        assert!(matches!(table.columns[0].data_type, ColumnDataType::Number));
        assert_eq!(table.columns[1].name, "name");
        assert!(matches!(table.columns[1].data_type, ColumnDataType::Text));

        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].values.len(), 2);
        assert!(matches!(table.rows[0].values[0], RowValue::Number(1)));
        assert!(matches!(&table.rows[0].values[1], RowValue::Text(value) if value == "test"));
        assert_eq!(table.rows[1].values.len(), 2);
        assert!(matches!(table.rows[1].values[0], RowValue::Number(2)));
        assert!(matches!(&table.rows[1].values[1], RowValue::Text(value) if value == "test_2"));
    }

    #[test]
    fn query_returns_parser_error_for_invalid_sql() {
        let mut database = Database::new("test_database");

        let result = run_query(&mut database, "SELECT FROM users;");

        assert!(matches!(
            result,
            Err(QueryError::Parse(ParseSqlErrors::MissingStartStatement))
        ));
    }

    #[test]
    fn query_returns_tokenizer_error_for_invalid_identifier() {
        let mut database = Database::new("test_database");

        let result = run_query(&mut database, "SELECT * FROM 2users;");

        assert!(matches!(
            result,
            Err(QueryError::Tokenize(TokenizeError::InvalidIdentifier(name))) if name == "2users"
        ));
    }

    #[test]
    fn query_returns_executor_error_for_missing_table() {
        let mut database = Database::new("test_database");

        let result = run_query(&mut database, "SELECT * FROM users;");

        assert!(matches!(
            result,
            Err(QueryError::Execute(ExecuteError::MissingTable))
        ));
    }

    #[test]
    fn query_rejects_trailing_tokens() {
        let mut database = Database::new("test_database");

        let result = run_query(&mut database, "SELECT * FROM users; extra");

        assert!(matches!(
            result,
            Err(QueryError::Parse(ParseSqlErrors::UnexpectedStatement))
        ));
    }

    #[test]
    fn query_creates_tables_with_different_numbers_of_columns() {
        for (sql, names) in [
            ("CREATE TABLE users (id NUMBER);", vec!["id"]),
            (
                "create table users(id number,name text)",
                vec!["id", "name"],
            ),
            (
                "CREATE TABLE users (id NUMBER, name TEXT, age NUMBER, city TEXT);",
                vec!["id", "name", "age", "city"],
            ),
        ] {
            let mut database = Database::new("test_database");

            assert!(matches!(
                run_query(&mut database, sql),
                Ok(QueryResult::TableCreated)
            ));

            let table = match run_query(&mut database, "SELECT * FROM users;") {
                Ok(QueryResult::Selected(table)) => table,
                _ => panic!("expected the created table"),
            };
            assert_eq!(
                table
                    .columns
                    .iter()
                    .map(|column| column.name.as_str())
                    .collect::<Vec<_>>(),
                names
            );
            assert!(matches!(table.columns[0].data_type, ColumnDataType::Number));
            if table.columns.len() > 1 {
                assert!(matches!(table.columns[1].data_type, ColumnDataType::Text));
            }
            assert!(table.rows.is_empty());
        }
    }

    #[test]
    fn query_rejects_duplicate_columns_without_creating_table() {
        let mut database = Database::new("test_database");

        let result = run_query(&mut database, "CREATE TABLE users (id NUMBER, id TEXT);");

        assert!(
            matches!(result, Err(QueryError::Execute(ExecuteError::DuplicateColumn(name))) if name == "id")
        );
        assert!(database.get_table("users").is_none());
    }

    #[test]
    fn query_rejects_duplicate_table_without_changing_original() {
        let mut database = Database::new("test_database");
        assert!(matches!(
            run_query(&mut database, "CREATE TABLE users (id NUMBER)"),
            Ok(QueryResult::TableCreated)
        ));
        database.get_table_mut("users").unwrap().rows.push(Row {
            values: vec![RowValue::Number(7)],
        });

        let result = run_query(&mut database, "CREATE TABLE users (name TEXT);");

        assert!(matches!(
            result,
            Err(QueryError::Execute(ExecuteError::TableNameExists))
        ));
        let table = database.get_table("users").unwrap();
        assert_eq!(table.columns.len(), 1);
        assert_eq!(table.columns[0].name, "id");
        assert!(matches!(table.columns[0].data_type, ColumnDataType::Number));
        assert_eq!(table.rows.len(), 1);
        assert!(matches!(table.rows[0].values[0], RowValue::Number(7)));
    }

    #[test]
    fn query_rejects_malformed_creation_without_modifying_database() {
        let mut database = Database::new("test_database");
        for sql in [
            "CREATE TABLE users ()",
            "CREATE TABLE users (id UNKNOWN)",
            "CREATE TABLE users (id NUMBER,)",
            "CREATE TABLE users (id NUMBER PRIMARY KEY)",
            "CREATE TABLE users (id NUMBER); extra",
        ] {
            assert!(
                matches!(run_query(&mut database, sql), Err(QueryError::Parse(_))),
                "accepted {sql}"
            );
            assert!(database.get_table("users").is_none());
        }
    }

    #[test]
    fn query_preserves_case_sensitive_names() {
        let mut database = Database::new("test_database");
        assert!(matches!(
            run_query(&mut database, "CREATE TABLE Users (id NUMBER, ID TEXT);"),
            Ok(QueryResult::TableCreated)
        ));
        assert!(matches!(
            run_query(&mut database, "SELECT * FROM Users"),
            Ok(QueryResult::Selected(_))
        ));
        assert!(matches!(
            run_query(&mut database, "SELECT * FROM users"),
            Err(QueryError::Execute(ExecuteError::MissingTable))
        ));
    }

    #[test]
    fn query_creates_table_with_many_columns() {
        let mut database = Database::new("test_database");
        let columns = (0..64)
            .map(|index| format!("column_{index} NUMBER"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("CREATE TABLE numbers ({columns});");

        assert!(matches!(
            run_query(&mut database, &sql),
            Ok(QueryResult::TableCreated)
        ));
        let table = database.get_table("numbers").unwrap();
        assert_eq!(table.columns.len(), 64);
        assert_eq!(table.columns[63].name, "column_63");
        assert!(table.rows.is_empty());
    }
}
