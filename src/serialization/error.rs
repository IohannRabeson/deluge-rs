use std::{num::ParseIntError, sync::Arc};

#[derive(thiserror::Error, Debug, Clone)]
pub enum SerializationError {
    #[error("parsing XML failed: {0}")]
    XmlParsingFailed(#[from] Arc<xmltree::ParseError>),

    #[error("parsing integer failed: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("parsing error: {0}")]
    SerdeError(#[from] serde_plain::Error),

    #[error("missing attribute '{1}' expected in parent '{0}'")]
    MissingAttribute(String, String),

    #[error("missing element '{0}'")]
    MissingElement(String),

    #[error("missing child '{1}' expected in parent '{0}")]
    MissingChild(String, String),

    #[error("unsupported sound source '{0}'")]
    UnsupportedSoundSource(String),

    #[error("unsupported sound type")]
    UnsupportedSoundType,

    #[error("invalid version format")]
    InvalidVersionFormat,

    #[error("overflow: {0} > {1}")]
    Overflow(String, String),

    #[error("underflow: {0} < {1}")]
    Underflow(String, String),

    #[error("invalid hexadecimal u32 '{0}': {1}")]
    ParseHexdecimalU32Error(String, std::num::ParseIntError),

    #[error("invalid i32 '{0}': {1}")]
    ParseI32Error(String, std::num::ParseIntError),

    #[error("conversion error: {0}")]
    ConversionError(#[from] Arc<std::io::Error>),

    #[error("unsupported modulation fx: {0}")]
    UnsupportedModulationFx(String),

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
