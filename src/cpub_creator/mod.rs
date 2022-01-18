mod metadata;
pub use metadata::Metadata;

pub use zip::write::ZipWriter;	
use std::io::Write;

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

        output.add_static_data()?;
        return Ok(output);
    }

    pub fn set_cover(&mut self) {

    }

    pub fn add_image(&mut self) {

    }

    pub fn close(&mut self) {
        
    }

    fn add_static_data(&mut self) -> Result<(), std::io::Error> {
        let options = zip::write::FileOptions::default();

        self.writer.start_file("mimetype", options.compression_method(zip::CompressionMethod::Stored))?;
        write!(self.writer, "application/epub+zip")?;

        self.writer.start_file("META-INF/container.xml", options)?;
        self.writer.write(b"lol")?;

        return Ok(());
    }
}
