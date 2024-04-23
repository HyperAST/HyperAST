use std::{fmt::Display, panic::Location};

#[derive(Debug, Clone)]
pub struct StringlyError(String);
impl Display for StringlyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for StringlyError {}

#[derive(Debug, Clone)]
pub struct LocatedError<E: std::error::Error + 'static> {
    inner: E,
    location: &'static Location<'static>,
}

impl<E: std::error::Error + 'static> std::error::Error for LocatedError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl<E: std::error::Error + 'static> std::fmt::Display for LocatedError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.inner, self.location)
    }
}

impl Into<LocatedError<std::io::Error>> for std::io::Error {
    #[track_caller]
    fn into(self) -> LocatedError<std::io::Error> {
        LocatedError {
            inner: self,
            location: std::panic::Location::caller(),
        }
    }
}
// impl From<std::io::Error> for LocatedError<std::io::Error> {
//     // The magic happens here
//     #[track_caller]
//     fn from(err: std::io::Error) -> Self {
//         LocatedError {
//             inner: err,
//             location: std::panic::Location::caller(),
//         }
//     }
// }

// TODO issue with auto implemented Into, not using #[track_caller]

// impl From<&str> for LocatedError<StringlyError> {
//     #[track_caller]
//     fn from(err: &str) -> Self {
//         LocatedError {
//             inner: StringlyError(err.to_string()),
//             location: std::panic::Location::caller(),
//         }
//     }
// }

impl Into<LocatedError<StringlyError>> for &str {
    #[track_caller]
    fn into(self) -> LocatedError<StringlyError> {
        LocatedError {
            inner: StringlyError(self.to_string()),
            location: std::panic::Location::caller(),
        }
    }
}

impl Into<LocatedError<StringlyError>> for String {
    #[track_caller]
    fn into(self) -> LocatedError<StringlyError> {
        LocatedError {
            inner: StringlyError(self),
            location: std::panic::Location::caller(),
        }
    }
}
// impl From<String> for LocatedError<StringlyError> {
//     #[track_caller]
//     fn from(err: String) -> Self {
//         LocatedError {
//             inner: StringlyError(err),
//             location: std::panic::Location::caller(),
//         }
//     }
// }
