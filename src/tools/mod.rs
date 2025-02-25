use std::fmt::Debug;

pub fn log_error<E: Debug, T>(value_to_return: T) -> impl FnOnce(E) -> T {
    |e| {
        error!("{e:#?}");
        value_to_return
    }
}

pub fn log_error_and_message<E: Debug, T>(message: &str, value_to_return: T) -> impl FnOnce(E) -> T {
    move |e| {
        error!("{message}\n{e:#?}");
        value_to_return
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::{log_error, log_error_and_message};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn should_log_error_and_return_value() {
        init();

        let expected_return_value = "test";
        let result = log_error(expected_return_value)("This is an error.");

        assert_eq!(expected_return_value, result);
    }

    #[test]
    fn should_log_error_and_message_and_return_value() {
        init();

        let expected_message = "This is a test message";
        let expected_return_value = "This is a test return value";
        let result = log_error_and_message(expected_message, expected_return_value)("This is an error.");

        assert_eq!(expected_return_value, result);
    }

}