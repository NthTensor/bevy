#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

//! Provides errors for the Bevy Game Engine, in the form of the `Advice` trait and
//! the `Report` type.
//!
//! `Advice` adds extra information to the `Error` trait, such as sugested
//! fixes, and can be automatically converted into rich `Report`s which the bevy
//! runtime knows how to display to users.
//!
//! Engine and library code should write custom error types which implement
//! `Advice` and return `Result<T, E>` using those types.
//!
//! When receving errors from library code, user code should either
//!
//! Fallible systems should return `Result` (without any generics, it equivilent
//! to `Result<(), Report>`). Library errors that implement `Advice` will
//! automatically be converted into a `Report` by the `?` operator. Errors which
//! do not implement `Advice` (usually from non-bevy-specific libraries) can be
//! manually converted to a report using `.report()`. Systems can also create
//! ad-hoc reports with the `report!()` macro.
//!
//! The key difference between an error type implementing `Advice` and a `Report`
//! is that the former may be handled internally by a system, whereas a latter is
//! mostly intended for reporting.

extern crate alloc;

use alloc::boxed::Box;
use core::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    panic::Location,
};

#[cfg(feature = "backtrace")]
use backtrace::Backtrace;

#[cfg(feature = "trace")]
use tracing_error::SpanTrace;

pub use bevy_error_macros::Advice;

pub mod prelude {
    pub use crate::{Advice, IntoReport, Report, Result, SetAdvice, Severity};
}

/// This trait adds rich metadata to an `Error` so that it can be automatically
/// converted into a `Report` and returned to the bevy runtime for error reporting.
///
/// Ideally, libraries intended to interoperate with bevy should implement this
/// trait on all their error types.
pub trait Advice: Error {
    /// Returns the severity of this error.
    fn severity(&self) -> Severity {
        Severity::Error
    }

    /// Returns a code for this error.
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        None
    }

    /// Returns a help message for this error.
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        None
    }

    /// Returns a url relevant to this error.
    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        None
    }
}

/// The severity of a diganostic report. Used by the handler to determine the
/// appropreate response. Defaults to `Error`.
#[derive(Debug)]
pub enum Severity {
    /// Should not interupt, should not be reported. Not an issue.
    Expected,
    /// Should not interupt, may be reported. A potential issue.
    Alert,
    /// May interupt, should be reported. A potential failure.
    Warning,
    /// Should interupt, must be reported. A critical failure.
    Error,
}

/// A helper trait for modifying advice at runtime.
///
/// Note: This almost always returns wrapped type-errased errors (such as
/// `DynamicAdvice`). In library code, it is better to define static error types
/// that directly implement `Advice` instead of modifying existing advice at runtime.
pub trait SetAdvice {
    /// The type returned by the trait operations. This is usually `Self`, but
    /// for static error types it is the `DynamicAdvice` wrapper.
    type Output;

    /// Sets the severity for the error.
    fn with_severity(self, severity: Severity) -> Self::Output;

    /// Sets the code for the error.
    fn with_code<D: Display>(self, code: D) -> Self::Output;

    /// Sets the help message for the error.
    fn with_help<D: Display>(self, help: D) -> Self::Output;

    /// Sets the url for the error.
    fn with_url<D: Display>(self, url: D) -> Self::Output;
}

impl<A: Advice + Send + Sync + 'static> SetAdvice for A {
    type Output = DynamicAdvice;

    fn with_severity(self, severity: Severity) -> DynamicAdvice {
        DynamicAdvice {
            severity,
            ..self.into()
        }
    }

    fn with_code<D: Display>(self, code: D) -> DynamicAdvice {
        DynamicAdvice {
            code: Some(code.to_string()),
            ..self.into()
        }
    }

    fn with_help<D: Display>(self, help: D) -> DynamicAdvice {
        DynamicAdvice {
            help: Some(help.to_string()),
            ..self.into()
        }
    }

    fn with_url<D: Display>(self, url: D) -> DynamicAdvice {
        DynamicAdvice {
            url: Some(url.to_string()),
            ..self.into()
        }
    }
}

/// An standard error type that can be constructed at runtime.
#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for RuntimeError {}

/// An error type implementing `Advice` that can be created or edited at runtime.
#[derive(Debug)]
pub struct DynamicAdvice {
    /// The internal error type this adds advice for.
    pub error: Box<dyn Error + Send + Sync + 'static>,
    /// The severity of the error.
    pub severity: Severity,
    /// The error code.
    pub code: Option<String>,
    /// A help message or suggestion.
    pub help: Option<String>,
    /// A relevant url.
    pub url: Option<String>,
}

impl Display for DynamicAdvice {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl Error for DynamicAdvice {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}

impl<A: Advice + Send + Sync + 'static> From<A> for DynamicAdvice {
    fn from(advice: A) -> DynamicAdvice {
        DynamicAdvice {
            severity: advice.severity(),
            code: advice.code().map(|code| code.to_string()),
            help: advice.help().map(|help| help.to_string()),
            url: advice.url().map(|url| url.to_string()),
            error: Box::new(advice) as Box<_>,
        }
    }
}

impl DynamicAdvice {
    /// Creates a new version of an error with empty advice fields. The new
    /// advice can be modified directly, or through the `SetAdvice` trait.
    ///
    /// Note: Calling this on an type implementing `Advice` will errase the
    /// static information! Use `Into` or `From` to convert from a static error
    /// type implementing `Advice` to to a dynamic `DynamicAdvice`, or use
    /// `SetAdvice` directly on the static type.
    pub fn from_error<E: Error + Send + Sync + 'static>(error: E) -> DynamicAdvice {
        DynamicAdvice {
            error: Box::new(error) as Box<_>,
            severity: Severity::Error,
            code: None,
            help: None,
            url: None,
        }
    }
}

impl SetAdvice for DynamicAdvice {
    type Output = DynamicAdvice;

    fn with_severity(self, severity: Severity) -> DynamicAdvice {
        DynamicAdvice { severity, ..self }
    }

    fn with_code<D: Display>(self, code: D) -> DynamicAdvice {
        DynamicAdvice {
            code: Some(code.to_string()),
            ..self
        }
    }

    fn with_help<D: Display>(self, help: D) -> DynamicAdvice {
        DynamicAdvice {
            help: Some(help.to_string()),
            ..self
        }
    }

    fn with_url<D: Display>(self, url: D) -> DynamicAdvice {
        DynamicAdvice {
            url: Some(url.to_string()),
            ..self
        }
    }
}

/// A detailed heap-allocated error report.
struct ReportFrame {
    /// The diagnostic for this report.
    advice: DynamicAdvice,
    /// The location where this diagnostic was created.
    location: &'static Location<'static>,
    /// The call stack when this diagnostic was screated.
    #[cfg(feature = "backtrace")]
    backtrace: Option<Backtrace>,
    /// The the span stack when this diagnstic was created.
    #[cfg(feature = "trace")]
    spantrace: Option<SpanTrace>,
    /// The number of times this diagnostic has already been emitted.
    count: Option<usize>,
}

/// A report represents a generalized runtime exception that must be handled by
/// the bevy runtime executor, and may be reported to a developer or user.
///
/// Reports can be generated automatically from errors that implement `Advice`.
/// Errors that do not implement `Advice` can be manually converted into a
/// report using `.report()`.
///
/// The data for this type is allocated on the heap, and `Result<(), Report>`
/// takes up only a single word, so fits easily in registers.
pub struct Report(Box<ReportFrame>);

impl Report {
    #[track_caller]
    #[cold]
    fn new<E>(error: E) -> Self
    where
        E: Advice + Send + Sync + 'static,
    {
        error.into()
    }

    #[track_caller]
    #[cold]
    pub fn from_std<E>(error: E) -> Self
    where
        E: Advice + Send + Sync + 'static,
    {
        Report::from_dynamic(error)
    }

    #[track_caller]
    #[cold]
    pub fn from_dynamic<A>(advice: A) -> Self
    where
        A: Into<DynamicAdvice>,
    {
        Report(Box::new(ReportFrame {
            advice: advice.into(),
            location: Location::caller(),
            #[cfg(feature = "backtrace")]
            backtrace: None,
            #[cfg(feature = "trace")]
            spantrace: None,
            count: None,
        }))
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        return Debug::fmt(&self.0.advice.error, f);
    }
}

impl<A: Into<DynamicAdvice>> From<A> for Report {
    #[track_caller]
    #[cold]
    fn from(advice: A) -> Report {
        Report::from_dynamic(advice)
    }
}

impl SetAdvice for Report {
    type Output = Report;

    fn with_severity(mut self, severity: Severity) -> Report {
        self.0.advice = self.0.advice.with_severity(severity);
        self
    }

    fn with_code<D: Display>(mut self, code: D) -> Report {
        self.0.advice = self.0.advice.with_code(code);
        self
    }

    fn with_help<D: Display>(mut self, help: D) -> Report {
        self.0.advice = self.0.advice.with_help(help);
        self
    }

    fn with_url<D: Display>(mut self, url: D) -> Report {
        self.0.advice = self.0.advice.with_url(url);
        self
    }
}

/// A result type for use in fallible systems.
pub type Result<T = (), E = Report> = core::result::Result<T, E>;

pub trait IntoReport<T, E> {
    fn report(self) -> Result<T, Report>;
}

impl<T, E> IntoReport<T, E> for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    /// Converts an error into a report.
    ///
    /// Note: Errors implementing `Advice` are automatically converted into a
    /// `Report` by the `?` operator. Calling this functon on them will errase
    /// the static information provided by the `Advice` trait, and is not
    /// advised.
    #[track_caller]
    fn report(self) -> Result<T, Report> {
        self.map_err(|e| DynamicAdvice::from_error(e).into())
    }
}

impl<T, A: SetAdvice> SetAdvice for Result<T, A> {
    type Output = Result<T, A::Output>;

    fn with_severity(self, severity: Severity) -> Self::Output {
        self.map_err(|report| report.with_severity(severity))
    }

    fn with_code<D: Display>(self, code: D) -> Self::Output {
        self.map_err(|report| report.with_code(code))
    }

    fn with_help<D: Display>(self, help: D) -> Self::Output {
        self.map_err(|report| report.with_help(help))
    }

    fn with_url<D: Display>(self, url: D) -> Self::Output {
        self.map_err(|report| report.with_url(url))
    }
}
