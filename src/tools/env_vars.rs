pub fn retrieve_arg_value<'a>(arg: &'a str, arg_names: &[&str]) -> Option<&'a str> {
    for arg_name in arg_names {
        let arg_prefix = format!("{arg_name}=");
        if arg.starts_with(&arg_prefix) {
            return arg.split_once("=").map(|(_, l)| l);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use parameterized::{ide, parameterized};
    use crate::tools::env_vars::retrieve_arg_value;

    ide!();

    #[parameterized(
        arg = {"-l=test_login", "--login=test_login", "-p=test_password", "--password=test_password", "--another-arg=wrong"},
        arg_names = {& ["-l", "--login"], & ["-l", "--login"], & ["-p", "--password"], & ["-p", "--password"], & ["-p", "--password"]},
        expected_result = {Some("test_login"), Some("test_login"), Some("test_password"), Some("test_password"), None}
    )]
    fn should_retrieve_arg_value(arg: &str, arg_names: &[&str], expected_result: Option<&str>) {
        let result = retrieve_arg_value(arg, arg_names);
        assert_eq!(expected_result, result);
    }
}