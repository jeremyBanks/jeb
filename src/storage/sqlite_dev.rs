#![allow(unsafe_code)]
use std::sync::Arc;
use std::sync::Mutex;
use std::vec;

use rusqlite::LoadExtensionGuard;
use rusqlite_migration::Migrations;
use rusqlite_migration::M;
use tracing::error;
use tracing::info;
use tracing::instrument;
use tracing::trace;

use super::Storage;

#[derive(Debug, Clone)]
pub struct SqliteStorage {
    connection: Arc<Mutex<rusqlite::Connection>>,
}

impl Default for SqliteStorage {
    fn default() -> Self {
        Self {
            connection: Arc::new(Mutex::new(rusqlite::Connection::open_in_memory().unwrap())),
        }
    }
}

impl Storage for SqliteStorage {}

const APPLICATION_ID: u32 = 0x0F1C_1500;

impl SqliteStorage {
    #[instrument]
    pub fn new(mut connection: rusqlite::Connection) -> Result<Self, rusqlite::Error> {
        info!("Initializing connection...");

        info!("Loading sqlite_zstd extension...");

        unsafe {
            let guard = LoadExtensionGuard::new(&connection)?;
            connection.load_extension("sqlite_zstd", None)?;
        };

        let (application_id, user_version, schema_version) = connection.query_row(
            "select
                application_id ,
                user_version ,
                schema_version
            from
                pragma_application_id join
                pragma_user_version join
                pragma_schema_version",
            (),
            |row| {
                Ok((
                    row.get_ref(0)?.as_i64()? as u32,
                    row.get_ref(1)?.as_i64()? as u32,
                    row.get_ref(2)?.as_i64()? as u32,
                ))
            },
        )?;

        trace!(
            application_id = application_id,
            user_version = user_version,
            schema_version = schema_version
        );

        if application_id == 0 || application_id != APPLICATION_ID {
            if application_id == 0 {
                info!("initializing application_id to {APPLICATION_ID:08X}");
                connection.pragma_update(None, "application_id", APPLICATION_ID)?;
            }

            connection.pragma_update(None, "page_size", 64 * 1024)?;
            connection.pragma_update(None, "cache_size", -1 * 64 * 1024 * 1024)?;
            connection.pragma_update(None, "auto_vacuum", "full")?;
            connection.pragma_update(None, "foreign_keys", true)?;
            connection.pragma_update(None, "synchronous", "normal")?;
            connection.pragma_update(None, "temp_store", "memory")?;
            connection.pragma_update(None, "secure_delete", true)?;
            connection.pragma_update(None, "journal_mode", "wal")?;

            Migrations::new(vec![
                M::up(
                    r#"
                    create table BlobStore(
                        blip Blob primary key,
                        bytes Blob not null
                    ) strict;
                "#,
                ),
                M::up(
                    r#"
                    commit; -- rusqlite_migrations compatibility hack
                    select zstd_enable_transparent( '{
                        "table": "BlobStore",
                        "column": "bytes",
                        "compression_level": 21,
                        "dict_chooser": "''1''"
                    }');
                    begin transaction; -- rusqlite_migrations compatibility hack
                "#,
                ),
                M::up(
                    r#"
                    create table if not exists QueryCache(
                        request_blip Blob not null,
                        response_blip Blob not null,
                        first_seen Integer not null default( CURRENT_TIMESTAMP ),
                        last_seen Integer not null default( CURRENT_TIMESTAMP ),
                        unique( request_blip, response_blip ),
                        foreign key( request_blip ) references BlobStore( blip ),
                        foreign key( response_blip ) references BlobStore( blip )
                    ) strict;
                "#,
                ),
            ])
            .to_latest(&mut connection)
            .unwrap();
        } else {
            error!(
                "database has an unexpected application_id: {application_id:08X}. skipping \
                 migrations and pragmas."
            );
        }

        info!("optimizing database");
        connection.execute_batch(
            r#"
                select zstd_incremental_maintenance(null, 0.9375);
                vacuum;
                analyze;
            "#,
        )?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        Self::new(rusqlite::Connection::open_in_memory()?)
    }

    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        Self::new(rusqlite::Connection::open(path)?)
    }
}
