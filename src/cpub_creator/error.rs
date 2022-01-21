use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpubWriterError {
    /// Represents a failure to read from input.
    #[error("Invalid image error")]
    InvalidImageError { source: image::ImageError },

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
}