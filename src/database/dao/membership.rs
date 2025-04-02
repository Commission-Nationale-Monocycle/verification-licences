use crate::database::model::membership::Membership;
use diesel::{Connection, QueryDsl, RunQueryDsl, SelectableHelper};

pub fn get_all(connection: &mut impl Connection) {
    use crate::database::schema::membership::dsl::*;

    let results = membership
        .select(Membership::as_select())
        .load(connection)
        .expect("Error loading posts");

    println!("Displaying {} memberships", results.len());
    for membership in results {
        println!("{}", membership.id);
    }
}
