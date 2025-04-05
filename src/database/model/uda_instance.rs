use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Insertable, Debug, PartialEq)]
#[diesel(table_name = crate::database::schema::uda_instance)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub(crate) struct UdaInstance {
    id: i32,
    slug: String,
    name: String,
    url: String,
}

impl From<UdaInstance> for dto::uda_instance::Instance {
    fn from(value: UdaInstance) -> Self {
        dto::uda_instance::Instance::new(value.slug, value.name, value.url)
    }
}
