use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("unexpected token")]
    UnexpectedToken,

    #[error("missing library declaration")]
    MissingLibraryHeader,

    #[error("missing type header")]
    MissingTypeParameter,

    #[error("unknown primitive type")]
    UnknownPrimitiveType,
}