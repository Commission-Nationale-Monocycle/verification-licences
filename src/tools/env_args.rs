use std::cell::RefCell;
use std::env;

/// Retrieve value associated to an arg passed to the app.
///
/// /!\ As this works on global variables,
/// a function using `retrieve_arg_value` could be tricky to test.
/// To do so, wrap your test with `with_env_args(args, fn)`.
/// This function is only available in a test context.
pub fn retrieve_arg_value(arg_names: &[&str]) -> Option<String> {
    let args: Vec<String> = get_env_args();
    for arg in args {
        for arg_name in arg_names {
            let arg_prefix = format!("{arg_name}=");
            if arg.starts_with(&arg_prefix) {
                return arg.split_once("=").map(|(_, l)| l.to_owned());
            }
        }
    }

    None
}

#[cfg(not(test))]
fn get_env_args() -> Vec<String> {
    env::args().collect()
}

#[cfg(test)]
thread_local! {
    /// A mutable `Vec<String>` to host env args for tests.
    /// When a test is run with `with_env_args`,
    /// the inner `Vec` is set to whatever param is passed.
    /// It is then cleared to get back to a clean basis.
    static ENV_ARGS: RefCell<Vec<String>> = RefCell::new(vec![]);
}
#[cfg(test)]
fn get_env_args() -> Vec<String> {
    ENV_ARGS.with(|vec| vec.clone().into_inner())
}

#[cfg(test)]
/// When running tests, env args are sourced from within the app.
/// You can set them up from there by wrapping your test with this function.
pub fn with_env_args<F, T>(args: Vec<String>, function: F) -> T
    where F: FnOnce() -> T
{
    ENV_ARGS.with(|refcell| {
        refcell.replace(args);
        let result = function();
        refcell.replace(vec![]);
        result
    })
}

#[cfg(test)]
pub mod tests {
    use parameterized::{ide, parameterized};

    use crate::tools::env_args::{retrieve_arg_value, with_env_args};

    ide!();

    #[parameterized(
        args = {vec ! ["-l=test_login".to_owned()], vec ! ["--login=test_login".to_owned()], vec ! ["-p=test_password".to_owned()], vec ! ["--password=test_password".to_owned()], vec ! ["--another-arg=wrong".to_owned()]},
        arg_names = {& ["-l", "--login"], & ["-l", "--login"], & ["-p", "--password"], & ["-p", "--password"], & ["-p", "--password"]},
        expected_result = {Some("test_login".to_owned()), Some("test_login".to_owned()), Some("test_password".to_owned()), Some("test_password".to_owned()), None}
    )]
    fn should_retrieve_arg_value(args: Vec<String>, arg_names: &[&str], expected_result: Option<String>) {
        let result = with_env_args(args, || retrieve_arg_value(arg_names));
        assert_eq!(expected_result, result);
    }
}
