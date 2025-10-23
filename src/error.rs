use std::fmt;

/// The result type that is returned by `Injector` instances.
pub type InjectorResult<T> = Result<T, InjectorError>;

/// The error type that is returned by `Injector` instances.
#[derive(Debug, Clone, Copy)]
pub enum InjectorError {
    /// Returned when an `Injector` fails to open a process.
    OpenProcessFailed,
    /// Returned when an `Injector` fails to allocate memory in a remote process.
    RemoteAllocFailed,
    /// Returned when an `Injector` fails to write memory at a remote address.
    RemoteWriteFailed,
    /// Returned when an `Injector` fails to retrieve a remote module handle.
    GetModuleFailed,
    /// Returned when an `Injector` fails to retrieve a remote procedure address.
    GetProcedureFailed,
    /// Returned when an `Injector` fails to create a remote thread.
    CreateThreadFailed,
    /// Returned when an `Injector` fails to take a process snapshot.
    SnapshotFailed,
    /// Returned when an `Injector` fails to find any processes.
    NoProcesses
}

impl fmt::Display for InjectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InjectorError::*;

        write!(f, "{}", match self {
            OpenProcessFailed  => "Failed to open remote process",
            RemoteAllocFailed  => "Failed to allocate memory in remote process",
            RemoteWriteFailed  => "Failed to write memory at remote address",
            GetModuleFailed    => "Failed to get remote module handle",
            GetProcedureFailed => "Failed to get remote procedure address",
            CreateThreadFailed => "Failed to create remote thread",
            SnapshotFailed     => "Failed to take process snapshot",
            NoProcesses        => "No processes matching given name found"
        })
    }
}