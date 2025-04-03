use crate::database::error::DatabaseError::{ConnectionFailed, MissingDatabaseUrl};
use crate::database::migrations::run_migrations;
use crate::error::Result;
use crate::tools::env_args::retrieve_expected_arg_value;
#[cfg(test)]
use crate::tools::env_args::with_env_args;
use crate::tools::log_error_and_return;
#[cfg(test)]
use crate::tools::test::tests::temp_dir;
use diesel::{Connection, SqliteConnection};

pub(super) mod dao;
pub(crate) mod error;
mod migrations;
mod model;
mod schema;

pub fn init_db() -> Result<()> {
    let mut connection = establish_connection()?;
    Ok(run_migrations(&mut connection)?)
}

/// Establish a connection to the database whose URL is passed as an argument to the app (`--database-url`).
pub fn establish_connection() -> Result<SqliteConnection> {
    let database_url = retrieve_expected_arg_value("--database-url", MissingDatabaseUrl)?;
    let connection = SqliteConnection::establish(&database_url)
        .map_err(log_error_and_return(ConnectionFailed))?;
    Ok(connection)
}

#[allow(clippy::test_attr_in_doctest)]
#[cfg(test)]
/// In order for tests to work, they should connect to a temporary database.
/// To do so, they can use this function, which will provide a new database.
/// E.g.:
/// ```rust
/// #[test]
/// fn test() {
///     with_temp_database(|| {
///          let mut connection = establish_connection();
///          // Do something with this connection
///     });
/// }
/// ```
///
/// This can also be used for async tests. E.g.:
///``` rust
/// #[test]
/// fn test() {
///     async fn async_test() {
///         // Do something asynchronously
///     }
///     with_temp_database(|| Runtime::new().unwrap().block_on(async_test()));
/// }
/// ```
pub(crate) fn with_temp_database<F, T>(function: F) -> T
where
    F: FnOnce() -> T,
{
    with_env_args(
        vec![format!(
            "--database-url={}",
            temp_dir().join("database.db").to_str().unwrap()
        )],
        || {
            init_db().unwrap();
            function()
        },
    )
}
