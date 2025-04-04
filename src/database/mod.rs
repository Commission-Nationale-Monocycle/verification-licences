use crate::database::error::DatabaseError;
use crate::database::error::DatabaseError::MissingDatabaseUrl;
use crate::database::migrations::run_migrations;
use crate::tools::env_args::retrieve_expected_arg_value;
#[cfg(test)]
use crate::tools::env_args::with_env_args;
#[cfg(test)]
use crate::tools::test::tests::temp_dir;
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub(super) mod dao;
pub(crate) mod error;
mod migrations;
mod model;
mod schema;

pub type Result<T, E = DatabaseError> = std::result::Result<T, E>;

pub(crate) fn init_connection_pool() -> Result<Pool<ConnectionManager<SqliteConnection>>> {
    let database_url = retrieve_expected_arg_value("--database-url", MissingDatabaseUrl)?;
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .map_err(|error| DatabaseError::R2d2(error.to_string()))?;
    let mut connection = pool
        .get()
        .map_err(|error| DatabaseError::R2d2(error.to_string()))?;
    run_migrations(&mut connection)?;
    Ok(pool)
}

#[allow(clippy::test_attr_in_doctest)]
#[cfg(test)]
/// In order for tests to work, they should connect to a temporary database.
/// To do so, they can use this function, which will provide a new database.
/// E.g.:
/// ```rust
/// #[test]
/// fn test() {
///     with_temp_database(|pool| {
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
///     with_temp_database(|pool| Runtime::new().unwrap().block_on(async_test()));
/// }
/// ```
pub(crate) fn with_temp_database<F, T>(function: F) -> T
where
    F: FnOnce(Pool<ConnectionManager<SqliteConnection>>) -> T,
{
    with_env_args(
        vec![format!(
            "--database-url={}",
            temp_dir().join("database.db").to_str().unwrap()
        )],
        || {
            let pool = init_connection_pool().unwrap();
            function(pool)
        },
    )
}
