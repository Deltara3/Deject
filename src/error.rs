use std::fmt;

/// The result type that is returned by `Injector` instances.
pub type InjectorResult<T> = Result<T, InjectorError>;

/// The error type that is returned by `Injector` instances.
#[derive(Debug)]
pub enum InjectorError {
    /// Returned when an `Injector` fails to open a process.
    OpenProcessFailed,
    /// Returned when an `Injector` fails to create a SID from a string.
    SidCreationFailed,
    /// Returned when an `Injector` fails to get security info.
    GetSecurityInfoFailed,
    /// Returned when an `Injector` fails to get security info.
    SetSecurityInfoFailed
}

impl fmt::Display for InjectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InjectorError::*;

        write!(f, "{}", match self {
            OpenProcessFailed     => "Failed to open remote process",
            SidCreationFailed     => "Failed to create SID from string",
            GetSecurityInfoFailed => "Failed to get security info",
            SetSecurityInfoFailed => "Failed to set security info",
        })
    }
}