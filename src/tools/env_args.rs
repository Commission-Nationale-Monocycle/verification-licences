#[cfg(test)]
use rocket::tokio::runtime::Runtime;
#[cfg(test)]
use std::cell::RefCell;
#[cfg(not(test))]
use std::env;
use std::ops::Deref;

// region ArgName
/// Simple wrapper around a collection of strings.
/// Can be constructed automatically from &str & Vec<&str>.
/// Useful to handle args which can have multiple names and those which can have no more than one name.
pub struct ArgName<'a> {
    names: Vec<&'a str>,
}
impl<'a> From<&'a str> for ArgName<'a> {
    fn from(val: &'a str) -> Self {
        ArgName { names: vec![val] }
    }
}

impl<'a> From<Vec<&'a str>> for ArgName<'a> {
    fn from(val: Vec<&'a str>) -> Self {
        ArgName { names: val }
    }
}

impl<'a> Deref for ArgName<'a> {
    type Target = Vec<&'a str>;

    fn deref(&self) -> &Self::Target {
        &self.names
    }
}
// endregion

/// Retrieve value associated to an arg passed to the app.
///
/// /!\ As this works on global variables,
/// a function using `retrieve_arg_value` could be tricky to test.
/// To do so, wrap your test with `with_env_args(args, fn)`.
/// This function is only available in a test context.
pub fn retrieve_arg_value<'a, A>(arg_names: A) -> Option<String>
where
    A: Into<ArgName<'a>>,
{
    let args: Vec<String> = get_env_args();
    let arg_names = arg_names.into();
    for arg in args {
        for arg_name in arg_names.iter() {
            let arg_prefix = format!("{arg_name}=");
            if arg.starts_with(&arg_prefix) {
                return arg.split_once("=").map(|(_, l)| l.to_owned());
            }
        }
    }

    None
}

/// Retrieve an arg value
pub fn retrieve_expected_arg_value<E>(arg_name: &str, error_if_missing: E) -> Result<String, E> {
    retrieve_arg_value(arg_name).ok_or(error_if_missing)
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
    /// It is then reset to its previous state.
    static ENV_ARGS: RefCell<Vec<String>> = const { RefCell::new(vec![]) };
}
#[cfg(test)]
fn get_env_args() -> Vec<String> {
    ENV_ARGS.with(|vec| vec.clone().into_inner())
}

#[cfg(test)]
/// When running tests, env args are extended from within the app.
/// You can set them up from there by wrapping your test with this function.
pub fn with_env_args<F, T>(mut args: Vec<String>, function: F) -> T
where
    F: FnOnce() -> T,
{
    ENV_ARGS.with(|refcell| {
        let global_env_args = std::env::args().collect::<Vec<String>>();
        args.extend_from_slice(&global_env_args);
        let old_value = refcell.replace(args);
        let result = function();
        refcell.replace(old_value);
        result
    })
}

#[cfg(test)]
/// When running tests, env args are extended from within the app.
/// You can set them up from there by wrapping your test with this function.
pub fn with_env_args_async<F, T>(mut args: Vec<String>, function: F) -> T
where
    F: AsyncFnOnce() -> T,
{
    ENV_ARGS.with(|refcell| {
        let global_env_args = std::env::args().collect::<Vec<String>>();
        args.extend_from_slice(&global_env_args);
        let old_value = refcell.replace(args);
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(function());
        refcell.replace(old_value);
        result
    })
}

#[cfg(test)]
pub mod tests {
    use parameterized::{ide, parameterized};

    use crate::tools::env_args::{retrieve_arg_value, retrieve_expected_arg_value, with_env_args};

    ide!();

    #[parameterized(
        args = {vec!["-l=test_login".to_owned()], vec!["--login=test_login".to_owned()], vec!["-p=test_password".to_owned()], vec!["--password=test_password".to_owned()], vec!["--another-arg=wrong".to_owned()]},
        arg_names = {vec!["-l", "--login"], vec!["-l", "--login"], vec!["-p", "--password"], vec!["-p", "--password"], vec!["-p", "--password"]},
        expected_result = {Some("test_login".to_owned()), Some("test_login".to_owned()), Some("test_password".to_owned()), Some("test_password".to_owned()), None}
    )]
    fn should_retrieve_arg_value(
        args: Vec<String>,
        arg_names: Vec<&str>,
        expected_result: Option<String>,
    ) {
        let result = with_env_args(args, || retrieve_arg_value(arg_names));
        assert_eq!(expected_result, result);
    }

    #[test]
    fn should_retrieve_expected_arg_value() {
        let arg_name = "--arg-name";
        let arg_value = "arg-value";
        let error = "error!";
        let args = vec![format!("{arg_name}={arg_value}")];

        let result = with_env_args(args, || retrieve_expected_arg_value(arg_name, error)).unwrap();

        assert_eq!(arg_value, result);
    }

    #[test]
    fn should_fail_to_retrieve_expected_arg_value() {
        let arg_name = "arg-name";
        let error = "error!";

        let result = retrieve_expected_arg_value(arg_name, error).unwrap_err();

        assert_eq!(error, result);
    }
}
