mod metadata;
mod templates;

pub use metadata::Metadata;

pub use zip::write::ZipWriter;	
use std::io::Write;

pub struct EpubWriter {
    metadata: Metadata,
    closed: bool,
    writer: zip::ZipWriter<std::fs::File>
}

impl EpubWriter {
    pub fn new(path: &std::path::Path) -> Result<EpubWriter, std::io::Error> {
        let file = std::fs::File::create(path)?;
        
        let mut output = EpubWriter {
            metadata: Default::default(),
            closed: false,
            writer: zip::ZipWriter::new(file)
        };

        output.add_static_data()?;
        return Ok(output);
    }

    pub fn metadata(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    pub fn set_cover(&mut self) -> Result<(), std::io::Error> {
        return Ok(());
    }

    pub fn add_image(&mut self) -> Result<(), std::io::Error> {
        return Ok(());
    }

    pub fn close(&mut self) -> Result<(), std::io::Error> {
        if self.closed {
            return Ok(());
        }

        self.closed = true;
        self.writer.finish()?;
        return Ok(());
    }

    fn add_static_data(&mut self) -> Result<(), std::io::Error> {
        let options = zip::write::FileOptions::default();

        self.writer.start_file("mimetype", options.compression_method(zip::CompressionMethod::Stored))?;
        write!(self.writer, "application/epub+zip")?;

        self.writer.start_file("META-INF/container.xml", options)?;
        write!(self.writer, "{}", templates::CONTAINER_XML)?;

        return Ok(());
    }
}

impl Drop for EpubWriter {
    fn drop(&mut self) {
        self.close().expect("Unhandled I/O error on close");
    }
}