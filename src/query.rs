use crate::{
    database::Database,
    executor::{ExecuteError, execute_sql_statement},
    parser::{ParseSqlErrors, parse_sql},
    table::Table,
    tokenizer::{TokenizeError, tokenize},
};

pub(crate) enum QueryError {
    Tokenize(TokenizeError),
    Parse(ParseSqlErrors),
    Execute(ExecuteError),
}

pub(crate) fn run_query<'db>(database: &'db Database, sql: &str) -> Result<&'db Table, QueryError> {
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
        executor::ExecuteError,
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

        let database = Database {
            name: String::from("test_database"),
            tables,
        };

        let sql_sentence = "SELECT * FROM users;";

        let result = run_query(&database, sql_sentence);

        let table = match result {
            Ok(table) => table,
            Err(_) => panic!("expected the users table"),
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
        let database = Database::new("test_database");

        let result = run_query(&database, "SELECT FROM users;");

        assert!(matches!(
            result,
            Err(QueryError::Parse(ParseSqlErrors::MissingStartStatement))
        ));
    }

    #[test]
    fn query_returns_tokenizer_error_for_invalid_identifier() {
        let database = Database::new("test_database");

        let result = run_query(&database, "SELECT * FROM 2users;");

        assert!(matches!(
            result,
            Err(QueryError::Tokenize(TokenizeError::InvalidIdentifier(name))) if name == "2users"
        ));
    }

    #[test]
    fn query_returns_executor_error_for_missing_table() {
        let database = Database::new("test_database");

        let result = run_query(&database, "SELECT * FROM users;");

        assert!(matches!(
            result,
            Err(QueryError::Execute(ExecuteError::MissingTable))
        ));
    }

    #[test]
    fn query_rejects_trailing_tokens() {
        let database = Database::new("test_database");

        let result = run_query(&database, "SELECT * FROM users; extra");

        assert!(matches!(
            result,
            Err(QueryError::Parse(ParseSqlErrors::UnexpectedStatement))
        ));
    }
}
