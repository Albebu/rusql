use std::collections::HashSet;

use crate::{database::Database, statement::Statement, table::Table};

pub(crate) enum ExecuteError {
    MissingTable,
    TableNameExists,
    DuplicateColumn(String),
    EmptyColumnList,
}

pub(crate) enum QueryResult<'db> {
    Selected(&'db Table),
    TableCreated,
}

pub(crate) fn execute_sql_statement<'db>(
    database: &'db mut Database,
    statement: &Statement,
) -> Result<QueryResult<'db>, ExecuteError> {
    match statement {
        Statement::Select { table_name } => database
            .get_table(table_name)
            .map(QueryResult::Selected)
            .ok_or(ExecuteError::MissingTable),
        Statement::CreateTable {
            table_name,
            columns,
        } => {
            if columns.is_empty() {
                return Err(ExecuteError::EmptyColumnList);
            }
            let mut names = HashSet::new();
            for column in columns {
                if !names.insert(&column.name) {
                    return Err(ExecuteError::DuplicateColumn(column.name.clone()));
                }
            }
            let table = Table {
                name: table_name.clone(),
                columns: columns.clone(),
                rows: Vec::new(),
            };
            database
                .create_table(table)
                .map_err(|_| ExecuteError::TableNameExists)?;
            Ok(QueryResult::TableCreated)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        column::{Column, ColumnDataType},
        database::Database,
        executor::{ExecuteError, QueryResult, execute_sql_statement},
        row::{Row, RowValue},
        statement::Statement,
        table::Table,
    };

    fn get_statement() -> Statement {
        Statement::Select {
            table_name: String::from("users"),
        }
    }

    fn get_database() -> Database {
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
                rows: vec![Row {
                    values: vec![RowValue::Number(1), RowValue::Text(String::from("test"))],
                }],
            },
        );

        Database {
            name: String::from("test_database"),
            tables,
        }
    }

    #[test]
    fn executor_retrieves_all_data_from_table() {
        let statement = get_statement();
        let mut database = get_database();

        let result = execute_sql_statement(&mut database, &statement);

        let table = match result {
            Ok(QueryResult::Selected(table)) => table,
            _ => panic!("expected the users table"),
        };

        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.columns[0].name, "id");
        assert_eq!(table.columns[1].name, "name");
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].values.len(), 2);

        assert!(matches!(table.rows[0].values[0], RowValue::Number(1)));
        assert!(matches!(&table.rows[0].values[1], RowValue::Text(value) if value == "test"));
    }

    #[test]
    fn executor_retrieves_empty_table() {
        let statement = get_statement();
        let mut database = get_database();
        database.get_table_mut("users").unwrap().rows.clear();

        let result = execute_sql_statement(&mut database, &statement);

        let table = match result {
            Ok(QueryResult::Selected(table)) => table,
            _ => panic!("expected the empty users table"),
        };
        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 2);
        assert!(table.rows.is_empty());
    }

    #[test]
    fn executor_missing_table() {
        let mut database = get_database();
        let statement = Statement::Select {
            table_name: String::from("orders"),
        };

        let result = execute_sql_statement(&mut database, &statement);

        assert!(result.is_err());
    }

    #[test]
    fn executor_rejects_empty_column_list_without_creating_table() {
        let mut database = Database::new("test_database");
        let statement = Statement::CreateTable {
            table_name: String::from("users"),
            columns: vec![],
        };

        assert!(matches!(
            execute_sql_statement(&mut database, &statement),
            Err(ExecuteError::EmptyColumnList)
        ));
        assert!(database.get_table("users").is_none());
    }
}
