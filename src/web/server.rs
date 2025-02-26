use rocket::{Build, Rocket};

use crate::tools::env_args::retrieve_arg_value;
use crate::web::api::server::ApiServer;
use crate::web::frontend::server::FrontendServer;

const PORT_ENV_ARG: &str = "--port";
const DEFAULT_PORT: i32 = 8000;

pub trait Server {
    fn initialize_managed_states(&self, rocket_build: Rocket<Build>) -> Rocket<Build>;
    fn mount_routes(&self, rocket_build: Rocket<Build>) -> Rocket<Build>;
}

pub fn build_server() -> Rocket<Build> {
    let api_port = get_api_port();
    let rocket_build = rocket::build()
        .configure(rocket::Config::figment().merge(("port", api_port)));

    let servers: Vec<Box<dyn Server>> = vec![
        Box::new(ApiServer::new()),
        Box::new(FrontendServer::new())
    ];
    servers.iter()
        .fold(rocket_build, |rocket_build, server| server.mount_routes(server.initialize_managed_states(rocket_build)))
}

fn get_api_port() -> i32 {
    retrieve_arg_value(&[PORT_ENV_ARG])
        .map(|port| port.parse::<i32>().ok())
        .unwrap_or(None)
        .unwrap_or(DEFAULT_PORT)
}

#[cfg(test)]
mod tests {
    use crate::tools::env_args::with_env_args;
    use crate::web::server::get_api_port;

    const PORT_ENV_ARG: &str = "--port";
    const DEFAULT_PORT: i32 = 8000;

    #[test]
    fn should_get_custom_api_port() {
        let expected_api_port = 10;
        let api_port = with_env_args(vec![format!("{PORT_ENV_ARG}={expected_api_port}")], get_api_port);

        assert_eq!(expected_api_port, api_port);
    }

    #[test]
    fn should_get_default_api_port_when_wrong_type() {
        let api_port = with_env_args(vec![format!("{PORT_ENV_ARG}=doe")], get_api_port);

        assert_eq!(DEFAULT_PORT, api_port);
    }

    #[test]
    fn should_get_default_api_port_when_no_value() {
        let api_port = with_env_args(vec![format!("{PORT_ENV_ARG}=")], get_api_port);

        assert_eq!(DEFAULT_PORT, api_port);
    }

    #[test]
    fn should_get_default_api_port_when_no_arg() {
        let api_port = with_env_args(vec![], get_api_port);

        assert_eq!(DEFAULT_PORT, api_port);
    }
}
