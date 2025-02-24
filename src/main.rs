use crate::download::connect;

mod download;


fn main() {
    env_logger::init();
    connect();
}

