use std::{alloc::GlobalAlloc, fs::File, io::{Read, Seek, SeekFrom, Write}, time::Duration};

use axum::{http::StatusCode, response::{IntoResponse, Response}};
use formatcore::FormatError;
use tempfile::NamedTempFile;
use tracing::info;

pub fn write_code_to_tempfile(code: &str, suffix: &str) -> Result<NamedTempFile, formatcore::FormatError> {
    // Save the contents in a temporary file
    let mut temp = tempfile::NamedTempFile::with_suffix(suffix)?;
    temp.write_all(code.as_bytes())?;
    info!("Wrote to tempfile at {:?}", temp.path());
    Ok(temp)
}

pub fn read_formatted(mut codefile: NamedTempFile) -> Result<String, formatcore::FormatError> {
    let mut code = String::new();

    // Seek to start
    codefile.seek(SeekFrom::Start(0))
        .map_err(|e| FormatError::CannotReadFileContents(e))?;
    codefile.read_to_string(&mut code)
        .map_err(|e| FormatError::CannotReadFileContents(e))?;
    info!("Read contents from file at {:?}", codefile.path());
    Ok(code)
}

pub fn read_formatted_stdout(vec: Vec<u8>) -> Result<String, formatcore::FormatError> {
    let code = String::from_utf8(vec).map_err(|e| FormatError::FormatterOutputNotUTF8)?;
    return Ok(code)
}

#[cfg(feature = "rust")]
pub mod enabled_formatter {
    use std::{io::Write, process::Command};

    use formatcore::FormatError;

    use crate::format::read_formatted;
    use tracing::{info, instrument};

    use super::{read_formatted_stdout, write_code_to_tempfile};

    pub const FORMATTER: &'static str = "Rust";
    pub const SUFFIX: &'static str = ".rs";

    pub struct Formatter;
    impl formatcore::Formatter for Formatter {
        #[instrument]
        fn format(code: &str) -> Result<String, formatcore::FormatError> {
            let mut file_with_code = super::write_code_to_tempfile(code, SUFFIX)?;

            info!("Running formatter");
            let cmd = Command::new("rustfmt")
                .arg(file_with_code.path())
                .arg("--emit")
                .arg("files")
                .output()
                .map_err(|e| FormatError::FormatterFailed(e))?;
            if !cmd.status.success() {
                let err = String::from_utf8(cmd.stderr).map_err(|e| FormatError::FormatterOutputNotUTF8)?;
                return Err(FormatError::FormatterError(err))
            }

            let formatted = read_formatted(file_with_code)?;
            println!("{formatted}");

            Ok(formatted)
        }
    }
}

#[cfg(feature = "typescript")]
pub mod enabled_formatter {
    use std::{io::Write, process::Command};

    use formatcore::FormatError;

    use crate::format::read_formatted;
    use tracing::{info, instrument};

    use super::{read_formatted_stdout, write_code_to_tempfile};

    pub const FORMATTER: &'static str = "TypeScript";
    pub const SUFFIX: &'static str = ".ts";

    pub struct Formatter;
    impl formatcore::Formatter for Formatter {
        #[instrument]
        fn format(code: &str) -> Result<String, formatcore::FormatError> {
            let mut file_with_code = super::write_code_to_tempfile(code, SUFFIX)?;

            info!("Running formatter on {file_with_code:?}");
            let cmd = Command::new("prettier")
                .arg(file_with_code.path())
                .arg("--write")
                .output()
                .map_err(|e| FormatError::FormatterFailed(e))?;
            if !cmd.status.success() {
                let err = String::from_utf8(cmd.stderr).map_err(|e| FormatError::FormatterOutputNotUTF8)?;
                return Err(FormatError::FormatterError(err))
            }

            let formatted = read_formatted(file_with_code)?;
            println!("{formatted}");

            Ok(formatted)
        }
    }
}

#[cfg(feature = "php")]
pub mod enabled_formatter {
    use std::{io::Write, process::Command};

    use formatcore::FormatError;

    use crate::format::read_formatted;
    use tracing::{info, instrument};

    use super::{read_formatted_stdout, write_code_to_tempfile};

    pub const FORMATTER: &'static str = "PHP";
    pub const SUFFIX: &'static str = ".php";

    pub struct Formatter;
    impl formatcore::Formatter for Formatter {
        #[instrument]
        fn format(code: &str) -> Result<String, formatcore::FormatError> {
            let mut file_with_code = super::write_code_to_tempfile(code, SUFFIX)?;

            info!("Running formatter on {file_with_code:?}");
            let cmd = Command::new("php-cs-fixer")
                .arg("fix")
                .arg(file_with_code.path())
                .output()
                .map_err(|e| FormatError::FormatterFailed(e))?;
            if !cmd.status.success() {
                let err = String::from_utf8(cmd.stderr).map_err(|e| FormatError::FormatterOutputNotUTF8)?;
                return Err(FormatError::FormatterError(err))
            }

            let formatted = read_formatted(file_with_code)?;
            println!("{formatted}");

            Ok(formatted)
        }
    }
}