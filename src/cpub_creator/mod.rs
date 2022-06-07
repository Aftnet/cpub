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

impl PageImage {
    fn new(
        image_data: &[u8],
        nav_label: Option<String>,
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
            nav_label: nav_label,
            extension: imgtypeinfo.0,
            mime_type: imgtypeinfo.1,
            size: imgsize,
            spread: imgsize.0 > imgsize.1,
        });
    }

    pub fn image_file_name(&self) -> String {
        format!("{}{}", self.base_name, self.extension)
    }

    pub fn page_file_names(&self) -> Vec<String> {
        if self.spread {
            vec![
                self.page_spread_left_file_name(),
                self.page_spread_right_file_name(),
            ]
        } else {
            vec![self.page_regular_file_name()]
        }
    }

    pub fn generate_pages_xml(&self) -> Vec<(String, String)> {
        if self.spread {
            vec![
                (
                    self.page_spread_left_file_name(),
                    self.generate_page_xml(templates::PAGE_SPREAD_L_XML),
                ),
                (
                    self.page_spread_right_file_name(),
                    self.generate_page_xml(templates::PAGE_SPREAD_R_XML),
                ),
            ]
        } else {
            vec![(
                self.page_regular_file_name(),
                self.generate_page_xml(templates::PAGE_REGULAR_XML),
            )]
        }
    }

    fn page_regular_file_name(&self) -> String {
        format!("{}.xhtml", self.base_name)
    }

    fn page_spread_left_file_name(&self) -> String {
        format!("{}_L.xhtml", self.base_name)
    }

    fn page_spread_right_file_name(&self) -> String {
        format!("{}_L.xhtml", self.base_name)
    }

    fn generate_page_xml(&self, template: &str) -> String {
        template
            .replace("IMGW", &format!("{}", &self.size.0))
            .replace("IMGHW", &format!("{}", &self.size.0 / 2))
            .replace("IMGH", &format!("{}", &self.size.1))
            .replace("FILENAME", &self.image_file_name())
    }
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
        label: Option<String>,
    ) -> Result<(), error::EpubWriterError> {
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
                return Err(error::EpubWriterError::PageSortingError {
                    page_number: self.images.len() as u32,
                });
            }
        } else {
            self.spread_allowed = !self.spread_allowed;
        }

        let options = zip::write::FileOptions::default();
        self.inner
            .start_file(format!("OEBPS/{}", page_image.image_file_name()), options)?;
        self.inner.write_all(&buffer)?;

        let pages = page_image.generate_pages_xml();
        for i in pages {
            self.inner.start_file(format!("OEBPS/{}", &i.0), options)?;
            self.inner.write_all(&i.1.as_bytes())?;
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
