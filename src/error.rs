#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failure: {0}")]
    Failure(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),

    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Spdx(#[from] spdx_rs::error::SpdxError),

    #[error(transparent)]
    SpdxExpression(#[from] spdx_rs::models::SpdxExpressionError),

    #[error(transparent)]
    Minidom(#[from] minidom::Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}
