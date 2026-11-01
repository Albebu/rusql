use crate::column::{Column, ColumnDataType};
use crate::row::{Row, RowValue};

pub(crate) enum InsertRowError {
    WrongNumberOfValues,
    TypeMismatch,
}

pub(crate) struct Table {
    pub(crate) name: String,
    pub(crate) columns: Vec<Column>,
    pub(crate) rows: Vec<Row>,
}

impl Table {
    pub(crate) fn insert_row(&mut self, row: Row) -> Result<(), InsertRowError> {
        if row.values.len() != self.columns.len() {
            return Err(InsertRowError::WrongNumberOfValues);
        }

        for (i, cell) in row.values.iter().enumerate() {
            let column = &self.columns[i];

            match (&column.data_type, cell) {
                (ColumnDataType::Text, RowValue::Text(_)) => {}
                (ColumnDataType::Number, RowValue::Number(_)) => {}
                _ => return Err(InsertRowError::TypeMismatch),
            }
        }

        self.rows.push(row);
        Ok(())
    }

    pub(crate) fn get_all_rows(&self) -> &[Row] {
        &self.rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_table() -> Table {
        Table {
            name: String::from("test_table"),
            columns: vec![Column {
                name: String::from("name"),
                data_type: ColumnDataType::Text,
            }],
            rows: Vec::new(),
        }
    }

    fn create_two_column_table() -> Table {
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
            rows: Vec::new(),
        }
    }

    #[test]
    fn inserting_a_valid_row_adds_it_to_the_table() {
        let mut table = create_test_table();

        let result = table.insert_row(Row {
            values: vec![RowValue::Text(String::from("Some string"))],
        });

        assert!(result.is_ok());
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn inserting_a_value_with_the_wrong_type_does_not_insert_the_row_in_the_table() {
        let mut table = create_test_table();

        let result = table.insert_row(Row {
            values: vec![RowValue::Number(0)],
        });

        assert!(matches!(result, Err(InsertRowError::TypeMismatch)));
        assert_eq!(table.rows.len(), 0);
    }

    #[test]
    fn inserting_fewer_values_than_columns_does_not_insert_the_row_in_the_table() {
        let mut table = create_test_table();

        let result = table.insert_row(Row { values: vec![] });

        assert!(matches!(result, Err(InsertRowError::WrongNumberOfValues)));
        assert_eq!(table.rows.len(), 0);
    }

    #[test]
    fn inserting_more_values_than_columns_does_not_insert_the_row_in_the_table() {
        let mut table = create_test_table();

        let result = table.insert_row(Row {
            values: vec![
                RowValue::Text(String::from("Some string")),
                RowValue::Text(String::from("Some string")),
            ],
        });

        assert!(matches!(result, Err(InsertRowError::WrongNumberOfValues)));
        assert_eq!(table.rows.len(), 0);
    }

    #[test]
    fn inserting_a_number_into_a_number_column_adds_the_row() {
        let mut table = create_two_column_table();
        table.columns.pop();

        let result = table.insert_row(Row {
            values: vec![RowValue::Number(42)],
        });

        assert!(result.is_ok());
        assert_eq!(table.rows.len(), 1);
        assert!(matches!(table.rows[0].values[0], RowValue::Number(42)));
    }

    #[test]
    fn inserting_multiple_valid_values_preserves_their_order() {
        let mut table = create_two_column_table();

        let result = table.insert_row(Row {
            values: vec![RowValue::Number(1), RowValue::Text(String::from("Ada"))],
        });

        assert!(result.is_ok());
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].values.len(), 2);
        assert!(matches!(table.rows[0].values[0], RowValue::Number(1)));
        assert!(matches!(&table.rows[0].values[1], RowValue::Text(name) if name == "Ada"));
    }

    #[test]
    fn mismatch_in_a_later_column_does_not_insert_the_row() {
        let mut table = create_two_column_table();

        let result = table.insert_row(Row {
            values: vec![RowValue::Number(1), RowValue::Number(2)],
        });

        assert!(matches!(result, Err(InsertRowError::TypeMismatch)));
        assert!(table.rows.is_empty());
    }

    #[test]
    fn invalid_insertion_preserves_existing_rows() {
        let mut table = create_test_table();
        table.rows.push(Row {
            values: vec![RowValue::Text(String::from("Ada"))],
        });

        let result = table.insert_row(Row {
            values: vec![RowValue::Number(42)],
        });

        assert!(matches!(result, Err(InsertRowError::TypeMismatch)));
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].values.len(), 1);
        assert!(matches!(&table.rows[0].values[0], RowValue::Text(name) if name == "Ada"));
    }

    #[test]
    fn inserting_text_into_a_number_column_does_not_insert_the_row() {
        let mut table = create_two_column_table();
        table.columns.pop();

        let result = table.insert_row(Row {
            values: vec![RowValue::Text(String::from("wrong type"))],
        });

        assert!(matches!(result, Err(InsertRowError::TypeMismatch)));
        assert!(table.rows.is_empty());
    }

    #[test]
    fn get_all_rows_returns_an_empty_slice_for_an_empty_table() {
        let table = create_test_table();

        assert!(table.get_all_rows().is_empty());
    }

    #[test]
    fn get_all_rows_returns_all_rows_in_order() {
        let mut table = create_test_table();
        table.rows = vec![
            Row {
                values: vec![RowValue::Text(String::from("Ada"))],
            },
            Row {
                values: vec![RowValue::Text(String::from("Grace"))],
            },
        ];

        let rows = table.get_all_rows();
        assert_eq!(rows.len(), 2);
        assert!(matches!(&rows[0].values[0], RowValue::Text(name) if name == "Ada"));
        assert!(matches!(&rows[1].values[0], RowValue::Text(name) if name == "Grace"));
    }
}
