mod column;
mod database;
mod executor;
mod parser;
mod query;
mod row;
mod statement;
mod table;
mod tokenizer;

use std::io::{self, BufRead, IsTerminal, Write};

use crate::{
    database::Database,
    executor::{ExecuteError, QueryResult},
    parser::ParseSqlErrors,
    query::{QueryError, run_query},
    row::RowValue,
    table::{InsertRowError, Table},
    tokenizer::TokenizeError,
};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_cli(stdin.lock(), stdout.lock(), stdin.is_terminal())
}

// Accepting any buffered input and writable output also makes the loop testable in memory.
fn run_cli(mut input: impl BufRead, mut output: impl Write, interactive: bool) -> io::Result<()> {
    let mut database = Database::new("rusql");

    loop {
        if interactive {
            // ? returns the I/O error immediately if writing or flushing fails.
            write!(output, "rusql> ")?;
            output.flush()?;
        }

        let mut line = String::new();
        // Reading zero bytes means the input has reached end-of-file.
        if input.read_line(&mut line)? == 0 {
            break;
        }

        if line.trim().is_empty() {
            continue;
        }

        match run_query(&mut database, &line) {
            Ok(QueryResult::TableCreated) => writeln!(output, "Table created.")?,
            Ok(QueryResult::RowInserted) => writeln!(output, "Row inserted.")?,
            Ok(QueryResult::Selected(table)) => print_table(&mut output, table)?,
            Err(error) => {
                let (stage, message) = query_error_message(error);
                writeln!(output, "{stage} error: {message}")?;
            }
        }
    }

    Ok(())
}

fn query_error_message(error: QueryError) -> (&'static str, &'static str) {
    match error {
        QueryError::Tokenize(error) => (
            "Tokenizer",
            match error {
                TokenizeError::InvalidIdentifier(_) => "invalid identifier or character.",
                TokenizeError::UnterminatedString => "unterminated text literal.",
                TokenizeError::IntegerOutOfRange(_) => "integer is outside the supported range.",
            },
        ),
        QueryError::Parse(error) => (
            "Parser",
            match error {
                ParseSqlErrors::EmptyStatement => "statement is empty.",
                ParseSqlErrors::MissingSelectStatement => "expected SELECT, CREATE, or INSERT.",
                ParseSqlErrors::MissingStartStatement => "expected * after SELECT.",
                ParseSqlErrors::MissinFromStatement => "expected FROM after *.",
                ParseSqlErrors::MissingTable | ParseSqlErrors::ExpectedTableName => {
                    "expected a table name."
                }
                ParseSqlErrors::UnexpectedStatement => "unexpected tokens after statement.",
                ParseSqlErrors::ExpectedTableKeyword => "expected TABLE after CREATE.",
                ParseSqlErrors::ExpectedOpeningParen => "expected an opening parenthesis.",
                ParseSqlErrors::ExpectedColumnName => "expected a column name.",
                ParseSqlErrors::ExpectedColumnType => "expected a column type: TEXT or NUMBER.",
                ParseSqlErrors::ExpectedCommaOrClosingParen => {
                    "expected a comma or closing parenthesis."
                }
                ParseSqlErrors::ExpectedIntoKeyword => "expected INTO after INSERT.",
                ParseSqlErrors::ExpectedValuesKeyword => "expected VALUES after the table name.",
                ParseSqlErrors::ExpectedValue => "expected a text or integer value.",
            },
        ),
        QueryError::Execute(error) => (
            "Executor",
            match error {
                ExecuteError::MissingTable => "table not found.",
                ExecuteError::TableNameExists => "table already exists.",
                ExecuteError::DuplicateColumn(_) => "duplicate column name.",
                ExecuteError::EmptyColumnList => "a table requires at least one column.",
                ExecuteError::InvalidInsertRowCount => "INSERT requires exactly one row.",
                ExecuteError::InsertRow(InsertRowError::WrongNumberOfValues) => {
                    "value count does not match the table's columns."
                }
                ExecuteError::InsertRow(InsertRowError::TypeMismatch) => {
                    "value type does not match the column type."
                }
            },
        ),
    }
}

fn print_table(output: &mut impl Write, table: &Table) -> io::Result<()> {
    for (index, column) in table.columns.iter().enumerate() {
        if index > 0 {
            write!(output, " | ")?;
        }
        write!(output, "{}", column.name)?;
    }
    writeln!(output)?;

    for row in &table.rows {
        for (index, value) in row.values.iter().enumerate() {
            if index > 0 {
                write!(output, " | ")?;
            }
            match value {
                RowValue::Text(text) => write!(output, "{text}")?,
                RowValue::Number(number) => write!(output, "{number}")?,
            }
        }
        writeln!(output)?;
    }

    if table.rows.is_empty() {
        writeln!(output, "(0 rows)")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run_cli;
    use std::io::Cursor;

    #[test]
    fn cli_ignores_empty_lines_and_stops_at_end_of_input() {
        let input = Cursor::new("\n  \nSELECT * FROM users;\n");
        let mut output = Vec::new();

        run_cli(input, &mut output, false).unwrap();

        assert_eq!(output, b"Executor error: table not found.\n");
    }

    #[test]
    fn cli_shows_prompt_for_interactive_input() {
        let input = Cursor::new("");
        let mut output = Vec::new();

        run_cli(input, &mut output, true).unwrap();

        assert_eq!(output, b"rusql> ");
    }

    #[test]
    fn cli_keeps_database_between_queries_and_continues_after_errors() {
        let input = Cursor::new(
            "CREATE TABLE users (id NUMBER);\n\
             INSERT INTO users VALUES (1);\n\
             INSERT INTO users VALUES ('wrong type');\n\
             SELECT * FROM users;\n",
        );
        let mut output = Vec::new();

        run_cli(input, &mut output, false).unwrap();

        assert_eq!(
            output,
            b"Table created.\nRow inserted.\nExecutor error: value type does not match the column type.\nid\n1\n"
        );
    }

    #[test]
    fn cli_displays_columns_empty_tables_and_rows() {
        let input = Cursor::new(
            "CREATE TABLE users (id NUMBER, name TEXT);\n\
             SELECT * FROM users;\n\
             INSERT INTO users VALUES (1, 'Ada');\n\
             INSERT INTO users VALUES (-2, 'Grace');\n\
             SELECT * FROM users;\n",
        );
        let mut output = Vec::new();

        run_cli(input, &mut output, false).unwrap();

        assert_eq!(
            output,
            b"Table created.\nid | name\n(0 rows)\nRow inserted.\nRow inserted.\nid | name\n1 | Ada\n-2 | Grace\n"
        );
    }

    #[test]
    fn cli_reports_errors_from_each_query_stage() {
        let input = Cursor::new(
            "INSERT INTO users VALUES ('unfinished\n\
             INSERT users VALUES (1);\n\
             DELETE users;\n\
             UPDATE users SET id = 1;\n\
             SELECT * FROM users;\n\
             CREATE TABLE users (id NUMBER);\n\
             INSERT INTO users VALUES (1, 2);\n\
             SELECT * FROM users;\n",
        );
        let mut output = Vec::new();

        run_cli(input, &mut output, false).unwrap();

        assert_eq!(
            output,
            b"Tokenizer error: unterminated text literal.\n\
              Parser error: expected INTO after INSERT.\n\
              Parser error: expected SELECT, CREATE, or INSERT.\n\
              Tokenizer error: invalid identifier or character.\n\
              Executor error: table not found.\n\
              Table created.\n\
              Executor error: value count does not match the table's columns.\n\
              id\n(0 rows)\n"
        );
    }
}
