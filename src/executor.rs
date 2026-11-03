use crate::{
    database::Database,
    statement::{self, Statement},
    table::Table,
};

pub(crate) enum ExecuteError {
    MissingTable,
}

pub(crate) fn execute_sql_statement<'db>(
    database: &'db Database,
    statement: &Statement,
) -> Result<&'db Table, ExecuteError> {
    match statement {
        Statement::Select { table_name } => {
            match database.get_table(table_name) {
                Some(table) => {
                    return Ok(table);
                }
                None => return Err(ExecuteError::MissingTable),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        column::{Column, ColumnDataType},
        database::Database,
        executor::execute_sql_statement,
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
        let database = get_database();

        let result = execute_sql_statement(&database, &statement);

        let table = match result {
            Ok(table) => table,
            Err(_) => panic!("expected the users table"),
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

        let result = execute_sql_statement(&database, &statement);

        let table = match result {
            Ok(table) => table,
            Err(_) => panic!("expected the empty users table"),
        };
        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 2);
        assert!(table.rows.is_empty());
    }

    #[test]
    fn executor_missing_table() {
        let database = get_database();
        let statement = Statement::Select {
            table_name: String::from("orders"),
        };

        let result = execute_sql_statement(&database, &statement);

        assert!(result.is_err());
    }
}
