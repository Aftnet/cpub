mod error;
mod metadata;
mod templates;

use image::GenericImageView;
use metadata::Metadata;
use std::io::Write;
use std::vec::{Vec};

struct PageInfo {
    image_size: (u32, u32),
    spread: bool
}

pub struct EpubWriter {
    metadata: Metadata,
    pages: std::vec::Vec<PageInfo>,
    cover_added: bool,
    closed: bool,
    writer: zip::ZipWriter<std::fs::File>
}

impl EpubWriter {
    pub fn new(path: &std::path::Path) -> Result<EpubWriter, std::io::Error> {
        let file = std::fs::File::create(path)?;
        
        let mut output = EpubWriter {
            metadata: Default::default(),
            pages: std::vec::Vec::default(),
            cover_added: false,
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

    pub fn add_image(&mut self, image: &mut dyn std::io::Read) -> Result<(), error::EpubWriterError> {
        let mut buffer: Vec<u8> = Vec::new();
        image.read_to_end(&mut buffer)?;
        
        let img = image::load_from_memory(&buffer).map_err(|source| error::EpubWriterError::InvalidImageError { source })?;
        let imgsize = img.dimensions();
        self.pages.push(PageInfo {
            image_size: imgsize,
            spread: imgsize.0 > imgsize.1
        });
        let pageinfo = self.pages.last().unwrap();

        let options = zip::write::FileOptions::default();
        let filename = format!("S01P{:06}.png", self.pages.len());
        self.writer.start_file(format!("OEBPS/{}", &filename), options)?;
        self.writer.write_all(&buffer)?;

        let xml = templates::PAGE_REGULAR_XML.replace("IMGW", &format!("{}",pageinfo.image_size.0));
        let xml = xml.replace("IMGH", &format!("{}",pageinfo.image_size.1));
        let xml = xml.replace("FILENAME", &filename);
        let filename = format!("S01P{:06}.xhtml", self.pages.len());
        self.writer.start_file(format!("OEBPS/{}", &filename), options)?;
        self.writer.write_all(xml.as_bytes())?;

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