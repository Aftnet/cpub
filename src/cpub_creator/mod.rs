mod error;
mod metadata;
mod templates;

use image::GenericImageView;
use metadata::Metadata;
use std::fs::File;
use std::io::BufWriter;
use std::io::prelude::*;
use std::vec::{Vec};

struct PageInfo {
    image_size: (u32, u32),
    spread: bool
}

pub struct EpubWriter<W: Write + Seek> {
    metadata: Metadata,
    pages: std::vec::Vec<PageInfo>,
    cover_added: bool,
    closed: bool,
    inner: zip::ZipWriter<W>
}

impl<W: Write + Seek> EpubWriter<W> {
    pub fn new(inner: W) -> Result<EpubWriter<W>, std::io::Error> {
        let mut output = EpubWriter {
            metadata: Default::default(),
            pages: std::vec::Vec::default(),
            cover_added: false,
            closed: false,
            inner: zip::ZipWriter::new(inner)
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

    pub fn add_image<T: std::io::Read>(&mut self, image: &mut T) -> Result<(), error::EpubWriterError> {
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
        self.inner.start_file(format!("OEBPS/{}", &filename), options)?;
        self.inner.write_all(&buffer)?;

        let xml = templates::PAGE_REGULAR_XML.replace("IMGW", &format!("{}",pageinfo.image_size.0))
        .replace("IMGH", &format!("{}",pageinfo.image_size.1)).replace("FILENAME", &filename);
        let filename = format!("S01P{:06}.xhtml", self.pages.len());
        self.inner.start_file(format!("OEBPS/{}", &filename), options)?;
        self.inner.write_all(xml.as_bytes())?;

        return Ok(());
    }

    pub fn close(&mut self) -> Result<(), std::io::Error> {
        if self.closed {
            return Ok(());
        }

        self.closed = true;
        self.inner.finish()?;
        return Ok(());
    }

    fn add_static_data(&mut self) -> Result<(), std::io::Error> {
        let options = zip::write::FileOptions::default();

        self.inner.start_file("mimetype", options.compression_method(zip::CompressionMethod::Stored))?;
        write!(self.inner, "application/epub+zip")?;

        self.inner.start_file("META-INF/container.xml", options)?;
        write!(self.inner, "{}", templates::CONTAINER_XML)?;

        return Ok(());
    }
}

impl<W: Write + std::io::Seek> Drop for EpubWriter<W> {
    fn drop(&mut self) {
        self.close().expect("Unhandled I/O error on close");
    }
}

pub fn create_at(path: &std::path::Path) -> Result<EpubWriter<BufWriter<File>>, std::io::Error> {
    let f = File::create(path)?;
    let f = BufWriter::new(f);
    return EpubWriter::new(f);
}