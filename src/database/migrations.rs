use crate::database::error::DatabaseError;
use crate::error::Result;
use diesel::sqlite::Sqlite;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub(crate) fn run_migrations(
    connection: &mut impl MigrationHarness<Sqlite>,
) -> Result<(), DatabaseError> {
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}
