use crate::database::error::DatabaseError;

pub(crate) mod last_update;
pub(crate) mod membership;
mod uda_instance;

type Result<T, E = DatabaseError> = std::result::Result<T, E>;
