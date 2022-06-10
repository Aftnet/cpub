mod errors;
mod metadata;
mod pageimage;
mod templates;

pub use metadata::Metadata;
use pageimage::PageImage;
use std::io::{prelude::*, Cursor};
use std::vec::Vec;
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};
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

    pub fn set_cover<T: std::io::Read>(&mut self, image: &mut T) -> Result<(), EpubWriterError> {
        if self.cover_added {
            return Err(EpubWriterError::CoverAlreadySetError);
        }

        let mut buffer: Vec<u8> = Vec::new();
        image.read_to_end(&mut buffer)?;
        let mut page_image = PageImage::new(&buffer, None)?;
        if page_image.spread {
            return Err(EpubWriterError::CoverSizeError);
        }

        page_image.base_name = "S00-Cover".to_string();
        self.images.insert(0, page_image);
        let page_image = self.images.first().unwrap();

        let img_filename = page_image.image_file_name();
        let pages = page_image.generate_pages_xml(self.metadata.right_to_left);

        self.add_zip_entry(&format!("OEBPS/{}", &img_filename), &buffer)?;
        for i in pages {
            self.add_zip_entry(&format!("OEBPS/{}", &i.0), &i.1.as_bytes())?;
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
        if page_image.spread {
            if !self.spread_allowed {
                return Err(errors::EpubWriterError::PageSortingError {
                    page_number: self.images.len() as u32,
                });
            }
        } else {
            self.spread_allowed = !self.spread_allowed;
        }

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
        let img_filename = page_image.image_file_name();
        let pages = page_image.generate_pages_xml(self.metadata.right_to_left);

        self.add_zip_entry(&format!("OEBPS/{}", &img_filename), &buffer)?;
        for i in pages {
            self.add_zip_entry(&format!("OEBPS/{}", &i.0), &i.1.as_bytes())?;
        }

        return Ok(());
    }

    pub fn close(&mut self) -> Result<(), EpubWriterError> {
        if self.closed {
            return Ok(());
        }

        self.closed = true;
        self.add_dynamic_data()?;
        self.inner.finish()?;

        return Ok(());
    }

    fn add_static_data(&mut self) -> Result<(), EpubWriterError> {
        let options = zip::write::FileOptions::default();

        self.inner.start_file(
            "mimetype",
            options.compression_method(zip::CompressionMethod::Stored),
        )?;
        write!(self.inner, "application/epub+zip")?;

        self.add_zip_entry(
            "META-INF/container.xml",
            templates::CONTAINER_XML.as_bytes(),
        )?;

        return Ok(());
    }

    fn add_dynamic_data(&mut self) -> Result<(), EpubWriterError> {
        let xml = self.generate_content_opf()?;
        self.add_zip_entry("OEBPS/content.opf", &xml)?;

        //to remove
        let mut f = std::fs::File::create(std::path::Path::new("test.txt")).unwrap();
        f.write_all(&xml).unwrap();
        return Ok(());
    }

    fn generate_content_opf(&mut self) -> xml::writer::Result<Vec<u8>> {
        fn add_element<W: Write>(
            writer: &mut EventWriter<W>,
            name: &str,
            content: Option<&str>,
            attributes: Option<Vec<(&str, &str)>>,
        ) -> xml::writer::Result<()> {
            let mut xml_event = XmlEvent::start_element(name);
            if attributes.is_some() {
                for i in attributes.unwrap() {
                    xml_event = xml_event.attr(i.0, i.1);
                }
            }

            writer.write(xml_event)?;

            if content.is_some() {
                writer.write(XmlEvent::characters(content.unwrap()))?;
            }

            writer.write(XmlEvent::end_element())?;
            Ok(())
        }

        let mut buffer = Vec::<u8>::new();
        let mut xml_writer = EventWriter::new_with_config(
            Cursor::new(&mut buffer),
            EmitterConfig {
                perform_indent: true,
                ..Default::default()
            },
        );

        xml_writer.write(
            XmlEvent::start_element("package")
                .default_ns("http://www.idpf.org/2007/opf")
                .attr("version", "3.0")
                .attr("prefix", "rendition: http://www.idpf.org/vocab/rendition/# cpublib: https://github.com/Aftnet/CPubLib")
                .attr("unique-identifier", "bookid")
        )?;
        xml_writer.write(
            XmlEvent::start_element("metadata").ns("dc", "http://purl.org/dc/elements/1.1/"),
        )?;

        add_element(&mut xml_writer, "dc:type", Some("text"), None)?;
        add_element(
            &mut xml_writer,
            "dc:identifier",
            Some(&self.metadata.id),
            Some(vec![("id", "bookid")]),
        )?;

        add_element(
            &mut xml_writer,
            "meta",
            Some(&chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
            Some(vec![("property", "dcterms:modified")]),
        )?;
        add_element(
            &mut xml_writer,
            "meta",
            Some("pre-paginated"),
            Some(vec![("property", "rendition:layout")]),
        )?;

        xml_writer.write(XmlEvent::end_element())?;
        xml_writer.write(XmlEvent::end_element())?;
        return Ok(buffer);
    }

    fn add_zip_entry(&mut self, name: &str, data: &[u8]) -> Result<(), EpubWriterError> {
        let options = zip::write::FileOptions::default();
        self.inner.start_file(name, options)?;
        self.inner.write_all(&data)?;
        return Ok(());
    }
}

impl<W: Write + std::io::Seek> Drop for EpubWriter<W> {
    fn drop(&mut self) {
        self.close().expect("Unhandled I/O error on close");
    }
}
