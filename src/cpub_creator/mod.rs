mod metadata;
pub use metadata::Metadata;

pub use zip::write::ZipWriter;	

pub struct EpubWriter {
    pub metadata: Metadata,

    writer: zip::ZipWriter<std::fs::File>
}

impl EpubWriter {
    pub fn new(path: &std::path::Path) -> Result<EpubWriter, std::io::Error> {
        let mut file = std::fs::File::create(path)?;
        
        let mut output = EpubWriter {
            metadata: Default::default(),
            writer: zip::ZipWriter::new(file)
        };
        return Ok(output);
    }

    pub fn set_cover() {

    }

    pub fn add_image() {

    }

    pub fn close() {
        
    }
}
