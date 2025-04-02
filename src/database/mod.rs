use crate::database::error::DatabaseError::{ConnectionFailed, MissingDatabaseUrl};
use crate::database::migrations::run_migrations;
use crate::error::Result;
use crate::tools::env_args::retrieve_expected_arg_value;
use crate::tools::log_error_and_return;
use diesel::{Connection, SqliteConnection};

pub(crate) mod error;
mod migrations;
mod schema;

pub fn init_db() -> Result<()> {
    let database_url = retrieve_expected_arg_value("--database-url", MissingDatabaseUrl)?;
    let mut connection = SqliteConnection::establish(&database_url)
        .map_err(log_error_and_return(ConnectionFailed))?;

    Ok(run_migrations(&mut connection)?)
}
