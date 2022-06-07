use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpubWriterError {
    #[error("Unsupported image")]
    UnsupportedImageError,

    #[error("Invalid image")]
    InvalidImageError(#[from] image::ImageError),

    #[error("Spread not allowed at page {page_number}")]
    PageSortingError {
        page_number: u32,
    },

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
}
