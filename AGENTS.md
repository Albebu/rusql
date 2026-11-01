# Agent guidance for rusql

## Project purpose

rusql is a learning project for understanding database internals and developing deep Rust knowledge. Favor clear, educational implementations over production-scale complexity or opaque abstractions. Explain important design tradeoffs when they help the learning process.

## MVP scope

The MVP accepts a minimal subset of these SQL statements:

- `CREATE TABLE`, with simple column types; no indexes or references.
- `SELECT`, `INSERT`, `UPDATE`, and `DELETE`, with only the features needed for a minimal working database.

Do not assume full SQL compliance or add advanced database features unless requested.

## Working principles

- The user writes the code; do not write or modify code unless the user explicitly asks you to do so. Instead, provide guidance and explanations that help the user implement it.
- Keep the implementation understandable and aligned with the goal of learning how databases work internally. Avoid delegating core database behavior to an existing database engine.
- Use idiomatic Rust, explicit types and error handling where useful, and simple abstractions. Avoid unnecessary dependencies and complexity.
- Make focused changes consistent with the existing code and preserve work outside the requested scope.
- Add or update tests for behavior changes, including relevant error cases and unsupported syntax. Run the relevant tests and checks available in the project, and report any that could not be run.
- Document supported SQL syntax and noteworthy limitations as they are implemented. Keep documentation consistent with actual behavior.
