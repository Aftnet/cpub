use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0} field cannot be empty or whitespace", field)]
pub struct MetadataValidationError {
    pub field: &'static str,
}

#[derive(Error, Debug)]
pub enum EpubWriterError {
    #[error("Invalid metadata")]
    InvalidMetadataError(#[from] MetadataValidationError),

    #[error("At least one page is required for a valid epub")]
    NoPagesError,

    #[error("Cover already set")]
    CoverAlreadySetError,

    #[error("Cover cannot be wider than tall")]
    CoverSizeError,

    #[error("Unsupported image")]
    UnsupportedImageError,

    #[error("Invalid image")]
    InvalidImageError(#[from] image::ImageError),

    #[error("Spread not allowed at page {page_number}")]
    PageSortingError { page_number: u32 },

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    XmlWritingError(#[from] xml::writer::Error),

    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
}
