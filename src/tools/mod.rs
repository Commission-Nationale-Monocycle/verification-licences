use std::fmt::Debug;

pub fn log_message<E: Debug>(message: &str) -> impl FnOnce(E) {
    move |e| {
        error!("{message}\n{e:#?}");
    }
}

pub fn log_error_and_return<E: Debug, T>(value_to_return: T) -> impl FnOnce(E) -> T {
    |e| {
        error!("{e:#?}");
        value_to_return
    }
}

pub fn log_message_and_return<E: Debug, T>(message: &str, value_to_return: T) -> impl FnOnce(E) -> T {
    move |e| {
        error!("{message}\n{e:#?}");
        value_to_return
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::{log_error_and_return, log_message_and_return};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn should_log_error_and_return_value() {
        init();

        let expected_return_value = "test";
        let result = log_error_and_return(expected_return_value)("This is an error.");

        assert_eq!(expected_return_value, result);
    }

    #[test]
    fn should_log_error_and_message_and_return_value() {
        init();

        let expected_message = "This is a test message";
        let expected_return_value = "This is a test return value";
        let result = log_message_and_return(expected_message, expected_return_value)("This is an error.");

        assert_eq!(expected_return_value, result);
    }
}