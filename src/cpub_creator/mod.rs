mod error;
mod metadata;
mod templates;

use image::GenericImageView;
use image::ImageFormat;
use metadata::Metadata;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::vec::Vec;

struct PageImage {
    base_name: String,
    nav_label: Option<String>,
    extension: &'static str,
    mime_type: &'static str,
    size: (u32, u32),
    spread: bool,
}

pub struct EpubWriter<W: Write + Seek> {
    metadata: Metadata,
    images: std::vec::Vec<PageImage>,
    spread_allowed: bool,
    cover_added: bool,
    closed: bool,
    current_chapter_number: u32,
    current_page_number: u32,
    inner: zip::ZipWriter<W>,
}

impl<W: Write + Seek> EpubWriter<W> {
    pub fn new(inner: W) -> Result<EpubWriter<W>, std::io::Error> {
        let mut output = EpubWriter {
            metadata: Default::default(),
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

    pub fn metadata(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    pub fn set_cover(&mut self) -> Result<(), std::io::Error> {
        return Ok(());
    }

    pub fn add_image<T: std::io::Read>(
        &mut self,
        image: &mut T,
        label: &Option<&str>,
    ) -> Result<(), error::EpubWriterError> {
        let mut buffer: Vec<u8> = Vec::new();
        image.read_to_end(&mut buffer)?;
        let mut page_image = self.get_page_image_info(&buffer)?;

        if label.is_some() || self.current_page_number == 0 {
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
                return Err(error::EpubWriterError::PageSortingError {
                    page_number: self.images.len() as u32,
                });
            }
        } else {
            self.spread_allowed = !self.spread_allowed;
        }

        let options = zip::write::FileOptions::default();
        let image_filename = format!("OEBPS/{}{}", &page_image.base_name, &page_image.extension);
        self.inner
            .start_file(format!("OEBPS/{}", &image_filename), options)?;
        self.inner.write_all(&buffer)?;

        let xml = templates::PAGE_REGULAR_XML
            .replace("IMGW", &format!("{}", page_image.size.0))
            .replace("IMGHW", &format!("{}", page_image.size.0 / 2))
            .replace("IMGH", &format!("{}", page_image.size.1))
            .replace("FILENAME", &image_filename);
        let filename = format!("S01P{:06}.xhtml", self.images.len());
        self.inner
            .start_file(format!("OEBPS/{}", &filename), options)?;
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

    fn get_page_image_info(
        &mut self,
        image_data: &[u8],
    ) -> Result<PageImage, error::EpubWriterError> {
        let imgfmt = image::guess_format(&image_data)
            .map_err(|source| error::EpubWriterError::InvalidImageError(source))?;
        let imgtypeinfo = match imgfmt {
            ImageFormat::Bmp => (".bmp", "image/bmp"),
            ImageFormat::Jpeg => (".jpg", "image/jpeg"),
            ImageFormat::Png => (".png", "image/png"),
            _ => return Err(error::EpubWriterError::UnsupportedImageError),
        };

        let img = image::load_from_memory(&image_data)
            .map_err(|source| error::EpubWriterError::InvalidImageError(source))?;
        let imgsize = img.dimensions();
        return Ok(PageImage {
            base_name: String::new(),
            nav_label: None,
            extension: imgtypeinfo.0,
            mime_type: imgtypeinfo.1,
            size: imgsize,
            spread: imgsize.0 > imgsize.1,
        });
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
