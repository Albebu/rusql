use std::collections::HashMap;

use crate::table::Table;

pub(crate) enum CreateTableError {
    TableNameExists,
}

pub(crate) struct Database {
    pub(crate) name: String,
    pub(crate) tables: HashMap<String, Table>,
}

impl Database {
    pub(crate) fn new(database_name: &str) -> Database {
        Database {
            name: database_name.to_string(),
            tables: HashMap::new(),
        }
    }

    pub(crate) fn create_table(&mut self, table: Table) -> Result<(), CreateTableError> {
        if self.tables.contains_key(&table.name) {
            return Err(CreateTableError::TableNameExists);
        }

        self.tables.insert(table.name.clone(), table);
        Ok(())
    }

    pub(crate) fn get_table(&self, table_name: &str) -> Option<&Table> {
        self.tables.get(table_name)
    }

    pub(crate) fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table> {
        self.tables.get_mut(table_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{Column, ColumnDataType};
    use crate::row::{Row, RowValue};

    fn create_test_table(name: &str, column_name: &str, value: &str) -> Table {
        Table {
            name: String::from(name),
            columns: vec![Column {
                name: String::from(column_name),
                data_type: ColumnDataType::Text,
            }],
            rows: vec![Row {
                values: vec![RowValue::Text(String::from(value))],
            }],
        }
    }

    fn create_test_database() -> Database {
        Database {
            name: String::from("test_database"),
            tables: HashMap::new(),
        }
    }

    #[test]
    fn new_database_has_the_given_name_and_no_tables() {
        let database = Database::new("test_database");

        assert_eq!(database.name, "test_database");
        assert!(database.tables.is_empty());
    }

    #[test]
    fn create_table_adds_the_table_and_returns_ok() {
        let mut database = create_test_database();

        let result = database.create_table(create_test_table("users", "name", "Ada"));

        assert!(result.is_ok());
        assert_eq!(database.tables.len(), 1);
        let table = database.tables.get("users").expect("table should exist");
        assert_eq!(table.name, "users");
        assert_eq!(table.columns[0].name, "name");
        assert!(matches!(&table.rows[0].values[0], RowValue::Text(value) if value == "Ada"));
    }

    #[test]
    fn create_table_rejects_a_duplicate_without_replacing_the_original() {
        let mut database = create_test_database();
        database.tables.insert(
            String::from("users"),
            create_test_table("users", "name", "Ada"),
        );

        let result = database.create_table(create_test_table("users", "other", "Grace"));

        assert!(matches!(result, Err(CreateTableError::TableNameExists)));
        assert_eq!(database.tables.len(), 1);
        let table = database
            .tables
            .get("users")
            .expect("original table should exist");
        assert_eq!(table.columns[0].name, "name");
        assert!(matches!(&table.rows[0].values[0], RowValue::Text(value) if value == "Ada"));
    }

    #[test]
    fn create_table_does_not_modify_existing_tables() {
        let mut database = create_test_database();
        database.tables.insert(
            String::from("users"),
            create_test_table("users", "name", "Ada"),
        );

        let result = database.create_table(create_test_table("books", "title", "Rust"));

        assert!(result.is_ok());
        assert_eq!(database.tables.len(), 2);
        assert!(database.tables.contains_key("books"));
        let table = database
            .tables
            .get("users")
            .expect("original table should exist");
        assert_eq!(table.name, "users");
        assert_eq!(table.columns[0].name, "name");
        assert!(matches!(&table.rows[0].values[0], RowValue::Text(value) if value == "Ada"));
    }

    #[test]
    fn get_table_returns_an_existing_table() {
        let mut database = create_test_database();
        database.tables.insert(
            String::from("users"),
            create_test_table("users", "name", "Ada"),
        );

        let table = database.get_table("users").expect("table should exist");

        assert_eq!(table.name, "users");
        assert!(matches!(&table.rows[0].values[0], RowValue::Text(value) if value == "Ada"));
    }

    #[test]
    fn get_table_returns_none_for_a_missing_table() {
        let database = create_test_database();

        assert!(database.get_table("missing").is_none());
    }

    #[test]
    fn get_table_mut_returns_none_for_a_missing_table() {
        let mut database = create_test_database();

        assert!(database.get_table_mut("missing").is_none());
    }

    #[test]
    fn get_table_mut_returns_the_stored_table_for_modification() {
        let mut database = create_test_database();
        database.tables.insert(
            String::from("users"),
            create_test_table("users", "name", "Ada"),
        );

        let table = database.get_table_mut("users").expect("table should exist");
        table.rows.push(Row {
            values: vec![RowValue::Text(String::from("Grace"))],
        });

        let table = database.tables.get("users").expect("table should exist");
        assert_eq!(table.rows.len(), 2);
        assert!(matches!(&table.rows[0].values[0], RowValue::Text(value) if value == "Ada"));
        assert!(matches!(&table.rows[1].values[0], RowValue::Text(value) if value == "Grace"));
    }
}
