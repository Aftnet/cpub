mod errors;
mod metadata;
mod pageimage;
mod templates;

pub use metadata::Metadata;
use pageimage::PageImage;
use std::io::prelude::*;
use std::vec::Vec;
use zip::ZipWriter;

use self::errors::EpubWriterError;

pub struct EpubWriter<W: Write + Seek> {
    metadata: Metadata,
    images: std::vec::Vec<PageImage>,
    spread_allowed: bool,
    cover_added: bool,
    closed: bool,
    current_chapter_number: u32,
    current_page_number: u32,
    inner: ZipWriter<W>,
}

impl<W: Write + Seek> EpubWriter<W> {
    pub fn new(inner: W, metadata: Metadata) -> Result<EpubWriter<W>, EpubWriterError> {
        metadata.validate()?;

        let mut output = EpubWriter {
            metadata: metadata,
            images: Vec::default(),
            spread_allowed: false,
            cover_added: false,
            closed: false,
            current_chapter_number: 0,
            current_page_number: 0,
            inner: zip::ZipWriter::new(inner),
        };

        output.add_static_data()?;
        return Ok(output);
    }

    pub fn set_cover(&mut self) -> Result<(), EpubWriterError> {
        if self.cover_added {
            return Err(EpubWriterError::CoverAlreadySetError);
        }

        self.cover_added = true;
        return Ok(());
    }

    pub fn add_image<T: std::io::Read>(
        &mut self,
        image: &mut T,
        label: Option<String>,
    ) -> Result<(), EpubWriterError> {
        let mut buffer: Vec<u8> = Vec::new();
        image.read_to_end(&mut buffer)?;
        let mut page_image = PageImage::new(&buffer, label)?;
        if page_image.nav_label.is_some() || self.current_page_number == 0 {
            self.current_page_number += 1;
            self.current_chapter_number = 0;
        }
        self.current_chapter_number += 1;

        page_image.base_name = format!(
            "S01-C{:06}P{:06}",
            self.current_page_number, self.current_chapter_number
        );
        self.images.push(page_image);
        let page_image = self.images.last().unwrap();

        if page_image.spread {
            if !self.spread_allowed {
                return Err(errors::EpubWriterError::PageSortingError {
                    page_number: self.images.len() as u32,
                });
            }
        } else {
            self.spread_allowed = !self.spread_allowed;
        }

        EpubWriter::add_zip_entry(
            &mut self.inner,
            &format!("OEBPS/{}", page_image.image_file_name()),
            &buffer,
        )?;

        let pages = page_image.generate_pages_xml(self.metadata.right_to_left);
        for i in pages {
            EpubWriter::add_zip_entry(
                &mut self.inner,
                &format!("OEBPS/{}", &i.0),
                &i.1.as_bytes(),
            )?;
        }

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

        self.inner.start_file(
            "mimetype",
            options.compression_method(zip::CompressionMethod::Stored),
        )?;
        write!(self.inner, "application/epub+zip")?;

        self.inner.start_file("META-INF/container.xml", options)?;
        write!(self.inner, "{}", templates::CONTAINER_XML)?;

        return Ok(());
    }

    fn add_zip_entry(
        writer: &mut ZipWriter<W>,
        name: &str,
        data: &[u8],
    ) -> Result<(), EpubWriterError> {
        let options = zip::write::FileOptions::default();
        writer.start_file(name, options)?;
        writer.write_all(&data)?;
        return Ok(());
    }
}

impl<W: Write + std::io::Seek> Drop for EpubWriter<W> {
    fn drop(&mut self) {
        self.close().expect("Unhandled I/O error on close");
    }
}
