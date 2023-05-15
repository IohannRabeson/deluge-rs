use std::{num::ParseIntError, sync::Arc};

/// Serialization error.
#[derive(thiserror::Error, Debug, Clone)]
pub enum SerializationError {
    /// Parsing XML failed.
    #[error("parsing XML failed: {0}")]
    XmlParsingFailed(#[from] Arc<xmltree::ParseError>),

    /// Parsing an integer failed.
    #[error("parsing integer failed: {0}")]
    ParseIntError(#[from] ParseIntError),

    /// Serde error.
    #[error("parsing error: {0}")]
    SerdeError(#[from] serde_plain::Error),

    /// Missing XML attribute.
    #[error("missing attribute '{1}' expected in parent '{0}'")]
    MissingAttribute(String, String),

    /// Missing XML element.
    #[error("missing element '{0}'")]
    MissingElement(String),

    /// Missing XML child.
    #[error("missing child '{1}' expected in parent '{0}")]
    MissingChild(String, String),

    /// Unsupported sound source.
    #[error("unsupported sound source '{0}'")]
    UnsupportedSoundSource(String),

    /// Unsupported sound type.
    #[error("unsupported sound type")]
    UnsupportedSoundType,

    /// Invalid version format.
    #[error("invalid version format")]
    InvalidVersionFormat,

    /// Numeric overflow.
    #[error("overflow: {0} > {1}")]
    Overflow(String, String),

    /// Numeric underflow.
    #[error("underflow: {0} < {1}")]
    Underflow(String, String),

    /// Invalid hexadecimal u32.
    #[error("invalid hexadecimal u32 '{0}': {1}")]
    ParseHexdecimalU32Error(String, std::num::ParseIntError),

    /// Invalid hexadecimal i32.
    #[error("invalid i32 '{0}': {1}")]
    ParseI32Error(String, std::num::ParseIntError),

    /// Conversion error.
    #[error("conversion error: {0}")]
    ConversionError(#[from] Arc<std::io::Error>),

    /// Unsupported modulation FX.
    #[error("unsupported modulation fx: {0}")]
    UnsupportedModulationFx(String),

    /// Value not found in table.
    #[error("value not found in table: {0}")]
    ValueNotFoundInTable(u32),
}

#[cfg(test)]
mod tests {
    fn check_sync<T: Sync>() {
        // Does nothing
    }

    #[test]
    fn test_error_is_sync() {
        check_sync::<super::SerializationError>();
    }
}
