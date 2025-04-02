use crate::database::error::DatabaseError;

mod last_update;
mod membership;

type Result<T, E = DatabaseError> = std::result::Result<T, E>;
