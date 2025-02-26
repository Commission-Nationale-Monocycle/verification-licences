use std::sync::Mutex;
use rocket::{Build, Rocket};
use crate::api::members_controller;
use crate::api::members_state::MembersState;
use crate::member::config::MembersProviderConfig;
use crate::tools::env_args::retrieve_arg_value;

const API_PORT_ENV_ARG: &str = "--api-port";
const DEFAULT_API_PORT: i32 = 8001;

pub fn start_api_server(members_provider_config: MembersProviderConfig, members_state: MembersState) -> Rocket<Build> {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", get_api_port())))
        .manage(members_provider_config)
        .manage(Mutex::new(members_state))
        .mount("/", routes![members_controller::list_members, members_controller::update_members])
}

fn get_api_port() -> i32 {
    retrieve_arg_value(&[API_PORT_ENV_ARG])
        .map(|port| port.parse::<i32>().ok())
        .unwrap_or(None)
        .unwrap_or(DEFAULT_API_PORT)
}

#[cfg(test)]
mod tests {
    use crate::api::server::{DEFAULT_API_PORT, get_api_port};
    use crate::tools::env_args::with_env_args;

    #[test]
    fn should_get_custom_api_port() {
        let expected_api_port = 10;
        let api_port = with_env_args(vec![format!("--api-port={expected_api_port}")], get_api_port);

        assert_eq!(expected_api_port, api_port);
    }

    #[test]
    fn should_get_default_api_port_when_wrong_type() {
        let api_port = with_env_args(vec!["--api-port=doe".to_owned()], get_api_port);

        assert_eq!(DEFAULT_API_PORT, api_port);
    }

    #[test]
    fn should_get_default_api_port_when_no_value() {
        let api_port = with_env_args(vec!["--api-port=".to_owned()], get_api_port);

        assert_eq!(DEFAULT_API_PORT, api_port);
    }

    #[test]
    fn should_get_default_api_port_when_no_arg() {
        let api_port = with_env_args(vec![], get_api_port);

        assert_eq!(DEFAULT_API_PORT, api_port);
    }
}
