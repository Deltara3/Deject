/// Unwraps a `Result` or `Option` and executes a closure on failure.
#[macro_export]
macro_rules! catch_unwrap {
    // If the passed value is a Result, execute this case.
    ($value:expr, |$err:pat_param| $closure:expr) => {
        match $value {
            Ok(inner) => inner,
            Err($err) => $closure
        }
    };

    // If the passed value is an Option, execute this case.
    ($value:expr, || $closure:expr) => {
        match $value {
            Some(inner) => inner,
            None        => $closure
        }
    };

    // If neither case is satisified, produce a compilation error.
    ($value:expr $(, closure:expr)?) => {
        compile_error!("catch_unwrap! requires a Result or Option followed by a closure (|e| {...} or || {...})");
    }
}