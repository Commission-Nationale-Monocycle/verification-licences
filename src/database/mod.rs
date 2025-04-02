use crate::database::error::DatabaseError::{ConnectionFailed, MissingDatabaseUrl};
use crate::database::migrations::run_migrations;
use crate::error::Result;
use crate::tools::env_args::retrieve_expected_arg_value;
use crate::tools::log_error_and_return;
use diesel::{Connection, SqliteConnection};

mod dao;
pub(crate) mod error;
mod migrations;
mod model;
mod schema;

pub fn init_db() -> Result<()> {
    let database_url = retrieve_expected_arg_value("--database-url", MissingDatabaseUrl)?;
    let mut connection = SqliteConnection::establish(&database_url)
        .map_err(log_error_and_return(ConnectionFailed))?;

    Ok(run_migrations(&mut connection)?)
}

#[cfg(test)]
mod tests {
    use crate::database::error::DatabaseError::ConnectionFailed;
    use crate::database::migrations::MIGRATIONS;
    use crate::tools::log_error_and_return;
    use crate::tools::test::tests::temp_dir;
    use diesel::{Connection, SqliteConnection};
    use diesel_migrations::MigrationHarness;

    pub fn establish_connection() -> SqliteConnection {
        let temp_dir = temp_dir();
        let database_url = temp_dir.join("database.db").to_str().unwrap().to_string();
        let mut connection = SqliteConnection::establish(&database_url)
            .map_err(log_error_and_return(ConnectionFailed))
            .unwrap();

        connection.run_pending_migrations(MIGRATIONS).unwrap();

        connection
    }
}
