use crate::download::{connect, export_list, get_list};
use crate::import_members_list::import_from_file;

mod download;
mod import_members_list;


fn main() {
    env_logger::init();
    let client = connect().unwrap();
    get_list(&client);
    let filename = export_list(&client).unwrap();

    import_from_file(&filename);
}

