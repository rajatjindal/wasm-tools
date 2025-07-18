//! A Rust parser for the [WebAssembly Text format][wat]
//!
//! This crate contains a stable interface to the parser for the [WAT][wat]
//! format of WebAssembly text files. The format parsed by this crate follows
//! the [online specification][wat].
//!
//! # Examples
//!
//! Parse an in-memory string:
//!
//! ```
//! # fn foo() -> wat::Result<()> {
//! let wat = r#"
//!     (module
//!         (func $foo)
//!
//!         (func (export "bar")
//!             call $foo
//!         )
//!     )
//! "#;
//!
//! let binary = wat::parse_str(wat)?;
//! // ...
//! # Ok(())
//! # }
//! ```
//!
//! Parse an on-disk file:
//!
//! ```
//! # fn foo() -> wat::Result<()> {
//! let binary = wat::parse_file("./foo.wat")?;
//! // ...
//! # Ok(())
//! # }
//! ```
//!
//! ## Evolution of the WAT Format
//!
//! WebAssembly, and the WAT format, are an evolving specification. Features are
//! added to WAT, WAT changes, and sometimes WAT breaks. The policy of this
//! crate is that it will always follow the [official specification][wat] for
//! WAT files.
//!
//! Future WebAssembly features will be accepted to this parser **and they will
//! not require a feature gate to opt-in**. All implemented WebAssembly features
//! will be enabled at all times. Using a future WebAssembly feature in the WAT
//! format may cause breakage because while specifications are in development
//! the WAT syntax (and/or binary encoding) will often change. This crate will
//! do its best to keep up with these proposals, but breaking textual changes
//! will be published as non-breaking semver changes to this crate.
//!
//! ## Stability
//!
//! This crate is intended to be a very stable shim over the `wast` crate
//! which is expected to be much more unstable. The `wast` crate contains
//! AST data structures for parsing `*.wat` files and they will evolve was the
//! WAT and WebAssembly specifications evolve over time.
//!
//! This crate is currently at version 1.x.y, and it is intended that it will
//! remain here for quite some time. Breaking changes to the WAT format will be
//! landed as a non-semver-breaking version change in this crate. This crate
//! will always follow the [official specification for WAT][wat].
//!
//! [wat]: http://webassembly.github.io/spec/core/text/index.html

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str;
use wast::core::EncodeOptions;
use wast::lexer::{Lexer, TokenKind};
use wast::parser::{self, ParseBuffer};

#[doc(inline)]
pub use wast::core::GenerateDwarf;

/// Parses a file on disk as a [WebAssembly Text format][wat] file, or a binary
/// WebAssembly file
///
/// This function will read the bytes on disk and delegate them to the
/// [`parse_bytes`] function. For more information on the behavior of parsing
/// see [`parse_bytes`].
///
/// # Errors
///
/// For information about errors, see the [`parse_bytes`] documentation.
///
/// # Examples
///
/// ```
/// # fn foo() -> wat::Result<()> {
/// let binary = wat::parse_file("./foo.wat")?;
/// // ...
/// # Ok(())
/// # }
/// ```
///
/// [wat]: http://webassembly.github.io/spec/core/text/index.html
pub fn parse_file(file: impl AsRef<Path>) -> Result<Vec<u8>> {
    Parser::new().parse_file(file)
}

/// Parses in-memory bytes as either the [WebAssembly Text format][wat], or a
/// binary WebAssembly module.
///
/// This function will attempt to interpret the given bytes as one of two
/// options:
///
/// * A utf-8 string which is a `*.wat` file to be parsed.
/// * A binary WebAssembly file starting with `b"\0asm"`
///
/// If the input is a string then it will be parsed as `*.wat`, and then after
/// parsing it will be encoded back into a WebAssembly binary module. If the
/// input is a binary that starts with `b"\0asm"` it will be returned verbatim.
/// Everything that doesn't start with `b"\0asm"` will be parsed as a utf-8
/// `*.wat` file, returning errors as appropriate.
///
/// For more information about parsing wat files, see [`parse_str`].
///
/// # Errors
///
/// In addition to all of the errors that can be returned from [`parse_str`],
/// this function will also return an error if the input does not start with
/// `b"\0asm"` and is invalid utf-8. (failed to even try to call [`parse_str`]).
///
/// # Examples
///
/// ```
/// # fn foo() -> wat::Result<()> {
/// // Parsing bytes that are actually `*.wat` files
/// assert_eq!(&*wat::parse_bytes(b"(module)")?, b"\0asm\x01\0\0\0");
/// assert!(wat::parse_bytes(b"module").is_err());
/// assert!(wat::parse_bytes(b"binary\0file\0\that\0is\0not\0wat").is_err());
///
/// // Pass through binaries that look like real wasm files
/// assert_eq!(&*wat::parse_bytes(b"\0asm\x01\0\0\0")?, b"\0asm\x01\0\0\0");
/// # Ok(())
/// # }
/// ```
///
/// [wat]: http://webassembly.github.io/spec/core/text/index.html
pub fn parse_bytes(bytes: &[u8]) -> Result<Cow<'_, [u8]>> {
    Parser::new().parse_bytes(None, bytes)
}

/// Parses an in-memory string as the [WebAssembly Text format][wat], returning
/// the file as a binary WebAssembly file.
///
/// This function is intended to be a stable convenience function for parsing a
/// wat file into a WebAssembly binary file. This is a high-level operation
/// which does not expose any parsing internals, for that you'll want to use the
/// `wast` crate.
///
/// # Errors
///
/// This function can fail for a number of reasons, including (but not limited
/// to):
///
/// * The `wat` input may fail to lex, such as having invalid tokens or syntax
/// * The `wat` input may fail to parse, such as having incorrect syntactical
///   structure
/// * The `wat` input may contain names that could not be resolved
///
/// # Examples
///
/// ```
/// # fn foo() -> wat::Result<()> {
/// assert_eq!(wat::parse_str("(module)")?, b"\0asm\x01\0\0\0");
/// assert!(wat::parse_str("module").is_err());
///
/// let wat = r#"
///     (module
///         (func $foo)
///
///         (func (export "bar")
///             call $foo
///         )
///     )
/// "#;
///
/// let binary = wat::parse_str(wat)?;
/// // ...
/// # Ok(())
/// # }
/// ```
///
/// [wat]: http://webassembly.github.io/spec/core/text/index.html
pub fn parse_str(wat: impl AsRef<str>) -> Result<Vec<u8>> {
    Parser::default().parse_str(None, wat)
}

/// Parser configuration for transforming bytes into WebAssembly binaries.
#[derive(Default)]
pub struct Parser {
    #[cfg(feature = "dwarf")]
    generate_dwarf: Option<GenerateDwarf>,
    _private: (),
}

impl Parser {
    /// Creates a new parser with th default settings.
    pub fn new() -> Parser {
        Parser::default()
    }

    /// Indicates that DWARF debugging information should be generated and
    /// emitted by default.
    ///
    /// Note that DWARF debugging information is only emitted for textual-based
    /// modules. For example if a WebAssembly binary is parsed via
    /// [`Parser::parse_bytes`] this won't insert new DWARF information in such
    /// a binary. Additionally if the text format used the `(module binary ...)`
    /// form then no DWARF information will be emitted.
    #[cfg(feature = "dwarf")]
    pub fn generate_dwarf(&mut self, generate: GenerateDwarf) -> &mut Self {
        self.generate_dwarf = Some(generate);
        self
    }

    /// Equivalent of [`parse_file`] but uses this parser's settings.
    pub fn parse_file(&self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        self._parse_file(path.as_ref())
    }

    fn _parse_file(&self, file: &Path) -> Result<Vec<u8>> {
        let contents = std::fs::read(file).map_err(|err| Error {
            kind: Box::new(ErrorKind::Io {
                err,
                file: Some(file.to_owned()),
            }),
        })?;
        match self.parse_bytes(Some(file), &contents) {
            // If the result here is borrowed then that means that the input
            // `&contents` was itself already a wasm module. We've already got
            // an owned copy of that so return `contents` directly after
            // double-checking it is indeed the same as the `bytes` return value
            // here. That helps avoid a copy of `bytes` via something like
            // `Cow::to_owned` which would otherwise copy the bytes.
            Ok(Cow::Borrowed(bytes)) => {
                assert_eq!(bytes.len(), contents.len());
                assert_eq!(bytes.as_ptr(), contents.as_ptr());
                Ok(contents)
            }
            Ok(Cow::Owned(bytes)) => Ok(bytes),
            Err(mut e) => {
                e.set_path(file);
                Err(e)
            }
        }
    }

    /// Equivalent of [`parse_bytes`] but uses this parser's settings.
    ///
    /// The `path` argument is an optional path to use when error messages are
    /// generated.
    pub fn parse_bytes<'a>(&self, path: Option<&Path>, bytes: &'a [u8]) -> Result<Cow<'a, [u8]>> {
        if bytes.starts_with(b"\0asm") {
            return Ok(bytes.into());
        }
        match str::from_utf8(bytes) {
            Ok(s) => self._parse_str(path, s).map(|s| s.into()),
            Err(_) => Err(Error {
                kind: Box::new(ErrorKind::Custom {
                    msg: "input bytes aren't valid utf-8".to_string(),
                    file: path.map(|p| p.to_owned()),
                }),
            }),
        }
    }

    /// Equivalent of [`parse_str`] but uses this parser's settings.
    ///
    /// The `path` argument is an optional path to use when error messages are
    /// generated.
    pub fn parse_str(&self, path: Option<&Path>, wat: impl AsRef<str>) -> Result<Vec<u8>> {
        self._parse_str(path, wat.as_ref())
    }

    fn _parse_str(&self, path: Option<&Path>, wat: &str) -> Result<Vec<u8>> {
        let mut _buf = ParseBuffer::new(wat).map_err(|e| Error::cvt(e, wat, path))?;
        #[cfg(feature = "dwarf")]
        _buf.track_instr_spans(self.generate_dwarf.is_some());
        let mut ast = parser::parse::<wast::Wat>(&_buf).map_err(|e| Error::cvt(e, wat, path))?;

        let mut _opts = EncodeOptions::default();
        #[cfg(feature = "dwarf")]
        if let Some(style) = self.generate_dwarf {
            _opts.dwarf(path.unwrap_or("<input>.wat".as_ref()), wat, style);
        }
        _opts
            .encode_wat(&mut ast)
            .map_err(|e| Error::cvt(e, wat, path))
    }
}

/// Result of [`Detect::from_bytes`] to indicate what some input bytes look
/// like.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Detect {
    /// The input bytes look like the WebAssembly text format.
    WasmText,
    /// The input bytes look like the WebAssembly binary format.
    WasmBinary,
    /// The input bytes don't look like WebAssembly at all.
    Unknown,
}

impl Detect {
    /// Detect quickly if supplied bytes represent a Wasm module,
    /// whether binary encoded or in WAT-encoded.
    ///
    /// This briefly lexes past whitespace and comments as a `*.wat` file to see if
    /// we can find a left-paren. If that fails then it's probably `*.wit` instead.
    ///
    ///
    /// Examples
    /// ```
    /// use wat::Detect;
    ///
    /// assert_eq!(Detect::from_bytes(r#"
    /// (module
    ///   (type (;0;) (func))
    ///   (func (;0;) (type 0)
    ///     nop
    ///   )
    /// )
    /// "#), Detect::WasmText);
    /// ```
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Detect {
        if bytes.as_ref().starts_with(b"\0asm") {
            return Detect::WasmBinary;
        }
        let text = match std::str::from_utf8(bytes.as_ref()) {
            Ok(s) => s,
            Err(_) => return Detect::Unknown,
        };

        let lexer = Lexer::new(text);
        let mut iter = lexer.iter(0);

        while let Some(next) = iter.next() {
            match next.map(|t| t.kind) {
                Ok(TokenKind::Whitespace)
                | Ok(TokenKind::BlockComment)
                | Ok(TokenKind::LineComment) => {}
                Ok(TokenKind::LParen) => return Detect::WasmText,
                _ => break,
            }
        }

        Detect::Unknown
    }

    /// Returns whether this is either binary or textual wasm.
    pub fn is_wasm(&self) -> bool {
        match self {
            Detect::WasmText | Detect::WasmBinary => true,
            Detect::Unknown => false,
        }
    }
}

/// A convenience type definition for `Result` where the error is [`Error`]
pub type Result<T> = std::result::Result<T, Error>;

/// Errors from this crate related to parsing WAT files
///
/// An error can during example phases like:
///
/// * Lexing can fail if the document is syntactically invalid.
/// * A string may not be utf-8
/// * The syntactical structure of the wat file may be invalid
/// * The wat file may be semantically invalid such as having name resolution
///   failures
#[derive(Debug)]
pub struct Error {
    kind: Box<ErrorKind>,
}

#[derive(Debug)]
enum ErrorKind {
    Wast(wast::Error),
    Io {
        err: std::io::Error,
        file: Option<PathBuf>,
    },
    Custom {
        msg: String,
        file: Option<PathBuf>,
    },
}

impl Error {
    fn cvt<E: Into<wast::Error>>(e: E, contents: &str, path: Option<&Path>) -> Error {
        let mut err = e.into();
        if let Some(path) = path {
            err.set_path(path);
        }
        err.set_text(contents);
        Error {
            kind: Box::new(ErrorKind::Wast(err)),
        }
    }

    /// To provide a more useful error this function can be used to set
    /// the file name that this error is associated with.
    ///
    /// The `file` here will be stored in this error and later rendered in the
    /// `Display` implementation.
    pub fn set_path<P: AsRef<Path>>(&mut self, file: P) {
        let file = file.as_ref();
        match &mut *self.kind {
            ErrorKind::Wast(e) => e.set_path(file),
            ErrorKind::Custom { file: f, .. } => *f = Some(file.to_owned()),
            ErrorKind::Io { file: f, .. } => *f = Some(file.to_owned()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.kind {
            ErrorKind::Wast(err) => err.fmt(f),
            ErrorKind::Custom { msg, file, .. } => match file {
                Some(file) => {
                    write!(f, "failed to parse `{}`: {}", file.display(), msg)
                }
                None => msg.fmt(f),
            },
            ErrorKind::Io { err, file, .. } => match file {
                Some(file) => {
                    write!(f, "failed to read from `{}`", file.display())
                }
                None => err.fmt(f),
            },
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &*self.kind {
            ErrorKind::Wast(_) => None,
            ErrorKind::Custom { .. } => None,
            ErrorKind::Io { err, .. } => Some(err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set_path() {
        let mut e = parse_bytes(&[0xFF]).unwrap_err();
        e.set_path("foo");
        assert_eq!(
            e.to_string(),
            "failed to parse `foo`: input bytes aren't valid utf-8"
        );

        let e = parse_file("_does_not_exist_").unwrap_err();
        assert!(
            e.to_string()
                .starts_with("failed to read from `_does_not_exist_`")
        );

        let mut e = parse_bytes("()".as_bytes()).unwrap_err();
        e.set_path("foo");
        assert_eq!(
            e.to_string(),
            "expected valid module field\n     --> foo:1:2\n      |\n    1 | ()\n      |  ^"
        );
    }
}
