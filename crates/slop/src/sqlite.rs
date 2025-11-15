//! SQLite integration for jeb
//!
//! This module provides SQLite custom functions and utilities for working with
//! JSON data using jeb's total ordering and binary encoding.

use {
    crate::to_sortable_bytes,
    rusqlite::{Connection, Result, functions::FunctionFlags},
    serde_json::Value,
};

/// Register jeb custom functions with a SQLite connection
///
/// This registers the `jeb_to_bytes(json_text)` function which converts
/// a JSON string to a sortable BLOB that preserves jeb's total ordering.
///
/// # Example
/// ```no_run
/// use {slop::sqlite::register_jeb_functions, rusqlite::Connection};
///
/// let conn = Connection::open_in_memory()?;
/// register_jeb_functions(&conn)?;
///
/// conn.execute(
///     "CREATE TABLE entities (
///         id INTEGER PRIMARY KEY,
///         json TEXT NOT NULL,
///         total_order BLOB GENERATED ALWAYS AS (jeb_to_bytes(json)) VIRTUAL
///     )",
///     [],
/// )?;
/// # Ok::<(), rusqlite::Error>(())
/// ```
pub fn register_jeb_functions(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "jeb_to_bytes",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let json_str = ctx.get::<String>(0)?;

            // Parse JSON
            let value: Value = serde_json::from_str(&json_str)
                .map_err(|e| rusqlite::Error::UserFunctionError(Box::new(e)))?;

            // Convert to sortable bytes
            Ok(to_sortable_bytes(&value))
        },
    )?;

    Ok(())
}

/// Create a jeb-enabled table with JSON storage and sortable index
///
/// Creates a table with:
/// - An `id` INTEGER PRIMARY KEY
/// - A `json` TEXT column for canonical JSON representation
/// - A `total_order` virtual generated column using `jeb_to_bytes(json)`
/// - An index on `total_order` for efficient range queries
///
/// # Arguments
/// * `conn` - SQLite connection (must have jeb functions registered)
/// * `table_name` - Name of the table to create
///
/// # Example
/// ```no_run
/// use {
///     slop::sqlite::{create_jeb_table, register_jeb_functions},
///     rusqlite::Connection,
/// };
///
/// let conn = Connection::open_in_memory()?;
/// register_jeb_functions(&conn)?;
/// create_jeb_table(&conn, "entities")?;
/// # Ok::<(), rusqlite::Error>(())
/// ```
pub fn create_jeb_table(conn: &Connection, table_name: &str) -> Result<()> {
    // Create table with generated column
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (
                id INTEGER PRIMARY KEY,
                json TEXT NOT NULL,
                total_order BLOB GENERATED ALWAYS AS (jeb_to_bytes(json)) VIRTUAL
            )"
        ),
        [],
    )?;

    // Create index on the generated column
    conn.execute(
        &format!(
            "CREATE INDEX IF NOT EXISTS idx_{table_name}_total_order ON {table_name}(total_order)"
        ),
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use {super::*, serde_json::json};

    #[test]
    fn test_register_jeb_functions() {
        let conn = Connection::open_in_memory().unwrap();
        register_jeb_functions(&conn).unwrap();

        // Test that the function works
        let result: Vec<u8> = conn
            .query_row(
                "SELECT jeb_to_bytes(?)",
                [r#"{"name":"Alice","id":1}"#],
                |row| row.get(0),
            )
            .unwrap();

        assert!(!result.is_empty());
        assert_eq!(result[0], b'{'); // Object type prefix
    }

    #[test]
    fn test_create_jeb_table() {
        let conn = Connection::open_in_memory().unwrap();
        register_jeb_functions(&conn).unwrap();
        create_jeb_table(&conn, "test_entities").unwrap();

        // Verify table exists with correct schema
        let table_info: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='test_entities'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(table_info.contains("total_order"));
        assert!(table_info.contains("GENERATED ALWAYS"));
    }

    #[test]
    fn test_sortable_ordering_in_sqlite() {
        let conn = Connection::open_in_memory().unwrap();
        register_jeb_functions(&conn).unwrap();
        create_jeb_table(&conn, "entities").unwrap();

        // Insert test data with different types
        let test_values = vec![
            json!({"type": "string", "value": "apple"}),
            json!({"type": "string", "value": "zebra"}),
            json!({"type": "number", "value": 42}),
            json!({"type": "number", "value": -10}),
            json!({"type": "array", "value": [1, 2, 3]}),
            json!({"type": "bool", "value": false}),
            json!({"type": "bool", "value": true}),
            json!({"type": "null", "value": null}),
        ];

        for value in &test_values {
            conn.execute("INSERT INTO entities (json) VALUES (?)", [
                serde_json::to_string(value).unwrap(),
            ])
            .unwrap();
        }

        // Query ordered by total_order
        let mut stmt = conn
            .prepare("SELECT json FROM entities ORDER BY total_order")
            .unwrap();

        let ordered_json: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Verify order: string < number < array < false < null < true < object
        assert_eq!(ordered_json.len(), 8);

        // Parse and verify ordering
        let ordered_values: Vec<Value> = ordered_json
            .iter()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();

        // Check that values are in correct type order
        for i in 1..ordered_values.len() {
            let prev = &ordered_values[i - 1];
            let curr = &ordered_values[i];
            assert!(
                crate::json_total_order(prev, curr) != std::cmp::Ordering::Greater,
                "Values out of order: {prev:?} > {curr:?}"
            );
        }
    }

    #[test]
    fn test_range_queries() {
        let conn = Connection::open_in_memory().unwrap();
        register_jeb_functions(&conn).unwrap();
        create_jeb_table(&conn, "entities").unwrap();

        // Insert numbers in random order
        for &n in &[5, 2, 8, 1, 9, 3, 7, 4, 6] {
            let json = json!(n);
            conn.execute("INSERT INTO entities (json) VALUES (?)", [
                serde_json::to_string(&json).unwrap(),
            ])
            .unwrap();
        }

        // Range query: find numbers between 3 and 7 (inclusive)
        let lower_bound = to_sortable_bytes(&json!(3));
        let upper_bound = to_sortable_bytes(&json!(7.99)); // Exclusive upper bound

        let mut stmt = conn
            .prepare(
                "SELECT json FROM entities
                 WHERE total_order >= ? AND total_order < ?
                 ORDER BY total_order",
            )
            .unwrap();

        let results: Vec<i64> = stmt
            .query_map([&lower_bound, &upper_bound], |row| {
                let json_str: String = row.get(0)?;
                let value: Value = serde_json::from_str(&json_str).unwrap();
                Ok(value.as_i64().unwrap())
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results, vec![3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_prefix_queries_objects() {
        let conn = Connection::open_in_memory().unwrap();
        register_jeb_functions(&conn).unwrap();
        create_jeb_table(&conn, "entities").unwrap();

        // Insert objects - Note: json! sorts keys alphabetically
        // So {"ns": "user", "id": 1} becomes {"id": 1, "ns": "user"} in the encoding
        // We use "ns" (comes after "id") to ensure it's last alphabetically
        let test_data = vec![
            json!({"id": 1, "ns": "user"}),
            json!({"id": 2, "ns": "user"}),
            json!({"id": 1, "ns": "admin"}),
            json!({"id": 1, "ns": "guest"}),
            json!({"other": "data"}),
        ];

        for value in test_data {
            conn.execute("INSERT INTO entities (json) VALUES (?)", [
                serde_json::to_string(&value).unwrap(),
            ])
            .unwrap();
        }

        // Find all objects starting with {"id": 1, "ns": "user"}
        // Note: Keys are alphabetically ordered, so "id" comes before "ns"
        let mut prefix_start = to_sortable_bytes(&json!({"id": 1, "ns": "user"}));

        // Remove the final terminator (\0\0) to allow prefix matching with additional
        // fields
        if prefix_start.len() >= 2 {
            prefix_start.truncate(prefix_start.len() - 2);
        }

        // Create upper bound by appending a high byte
        let mut prefix_end = prefix_start.clone();
        prefix_end.push(0xFF);

        let mut stmt = conn
            .prepare(
                "SELECT json FROM entities
                 WHERE total_order >= ? AND total_order < ?
                 ORDER BY total_order",
            )
            .unwrap();

        let results: Vec<Value> = stmt
            .query_map([&prefix_start, &prefix_end], |row| {
                let json_str: String = row.get(0)?;
                Ok(serde_json::from_str(&json_str).unwrap())
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Should find exactly the one object with id=1 and ns=user
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["id"], 1);
        assert_eq!(results[0]["ns"], "user");
    }

    #[test]
    fn test_type_filtering() {
        let conn = Connection::open_in_memory().unwrap();
        register_jeb_functions(&conn).unwrap();
        create_jeb_table(&conn, "entities").unwrap();

        // Insert mixed types
        let test_data = vec![
            json!("string1"),
            json!("string2"),
            json!(42),
            json!(100),
            json!([1, 2]),
            json!({"key": "value"}),
        ];

        for value in test_data {
            conn.execute("INSERT INTO entities (json) VALUES (?)", [
                serde_json::to_string(&value).unwrap(),
            ])
            .unwrap();
        }

        // Find only numbers (prefix '0')
        let number_prefix = vec![b'0'];
        let after_numbers = vec![b'[']; // Arrays come after numbers

        let mut stmt = conn
            .prepare(
                "SELECT json FROM entities
                 WHERE total_order >= ? AND total_order < ?
                 ORDER BY total_order",
            )
            .unwrap();

        let results: Vec<Value> = stmt
            .query_map([&number_prefix, &after_numbers], |row| {
                let json_str: String = row.get(0)?;
                Ok(serde_json::from_str(&json_str).unwrap())
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|v| v.is_number()));
    }
}
