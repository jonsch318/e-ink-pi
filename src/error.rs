//! Contains detailed error representation.
//!
//! See the main [`ImageError`] which contains a variant for each specialized error type. The
//! subtypes used in each variant are opaque by design. They can be roughly inspected through their
//! respective `kind` methods which work similar to `std::io::Error::kind`.
//!
//! The error interface makes it possible to inspect the error of an underlying decoder or encoder,
//! through the `Error::source` method. Note that this is not part of the stable interface and you
//! may not rely on a particular error value for a particular operation. This means mainly that
//! `image` does not promise to remain on a particular version of its underlying decoders but if
//! you ensure to use the same version of the dependency (or at least of the error type) through
//! external means then you could inspect the error type in slightly more detail.
//!
//! [`ImageError`]: enum.ImageError.html

use std::io;

//use crate::colors::ColorType;

/// The generic error type for image operations.
///
/// This high level enum allows, by variant matching, a rough separation of concerns between
/// underlying IO, the caller, format specifications, and the `image` implementation.
#[derive(Debug)]
pub enum ImageError {
    /// An error was encountered while decoding.
    ///
    /// This means that the input data did not conform to the specification of some image format,
    /// or that no format could be determined, or that it did not match format specific
    /// requirements set by the caller.
    Decoding,

    /// An error was encountered while encoding.
    ///
    /// The input image can not be encoded with the chosen format, for example because the
    /// specification has no representation for its color space or because a necessary conversion
    /// is ambiguous. In some cases it might also happen that the dimensions can not be used with
    /// the format.
    Encoding,

    /// An error was encountered in input arguments.
    ///
    /// This is a catch-all case for strictly internal operations such as scaling, conversions,
    /// etc. that involve no external format specifications.
    Parameter,

    /// Completing the operation would have required more resources than allowed.
    ///
    /// Errors of this type are limits set by the user or environment, *not* inherent in a specific
    /// format or operation that was executed.
    Limits,

    /// An operation can not be completed by the chosen abstraction.
    ///
    /// This means that it might be possible for the operation to succeed in general but
    /// * it requires a disabled feature,
    /// * the implementation does not yet exist, or
    /// * no abstraction for a lower level could be found.
    Unsupported,

    /// An error occurred while interacting with the environment.
    IoError(io::Error),
}
