use crate::download::{connect, export_list, get_list};

mod download;


fn main() {
    env_logger::init();
    let client = connect().unwrap();
    get_list(&client);
    let filename = export_list(&client).unwrap();
}

