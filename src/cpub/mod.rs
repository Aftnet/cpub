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
    images: Vec<PageImage>,
    cover: Option<PageImage>,
    spread_allowed: bool,
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
            cover: None,
            spread_allowed: false,
            closed: false,
            current_chapter_number: 0,
            current_page_number: 0,
            inner: zip::ZipWriter::new(inner),
        };

        output.add_static_data()?;
        return Ok(output);
    }

    pub fn set_cover<T: std::io::Read>(&mut self, image: &mut T) -> Result<(), EpubWriterError> {
        if self.cover.is_some() {
            return Err(EpubWriterError::CoverAlreadySetError);
        }

        let mut buffer: Vec<u8> = Vec::new();
        image.read_to_end(&mut buffer)?;
        let mut page_image = PageImage::new(&buffer, None)?;
        if page_image.spread {
            return Err(EpubWriterError::CoverSizeError);
        }

        page_image.base_name = "S00-Cover".to_string();
        self.cover = Some(page_image);
        let page_image = self.cover.as_ref().unwrap();

        let img_filename = page_image.image_file_name();
        self.add_zip_entry(&format!("OEBPS/{}", &img_filename), &buffer)?;

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
        for i in pages.iter() {
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
        if self.images.is_empty() {
            return Err(EpubWriterError::NoPagesError);
        }

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
            match attributes {
                Some(d) => {
                    for i in d.iter() {
                        xml_event = xml_event.attr(i.0, i.1);
                    }
                }
                None => {}
            }

            writer.write(xml_event)?;

            match content {
                Some(d) => writer.write(XmlEvent::characters(d))?,
                None => {}
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

        add_element(
            &mut xml_writer,
            "dc:title",
            Some(&self.metadata.title),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:creator",
            Some(&self.metadata.author),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:publisher",
            Some(&self.metadata.publisher),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:date",
            Some(&self.metadata.publishing_date.format("%Y-%m-%d").to_string()),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:language",
            Some(&self.metadata.language),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:description",
            Some(&self.metadata.description),
            None,
        )?;
        for i in self.metadata.tags.iter() {
            add_element(&mut xml_writer, "dc:subject", Some(i), None)?;
        }
        for i in self.metadata.custom.iter() {
            add_element(
                &mut xml_writer,
                "dc:subject",
                Some(i.1),
                Some(vec![("property", &format!("cpublib:{}", i.0))]),
            )?;
        }
        add_element(
            &mut xml_writer,
            "dc:source",
            Some(&self.metadata.source),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:relation",
            Some(&self.metadata.relation),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:rights",
            Some(&self.metadata.copyright),
            None,
        )?;

        xml_writer.write(XmlEvent::end_element())?;

        xml_writer.write(XmlEvent::start_element("manifest"))?;

        add_element(
            &mut xml_writer,
            "item",
            None,
            Some(vec![
                ("href", "nav.xhtml"),
                ("id", "nav"),
                ("media-type", "application/xhtml+xml"),
                ("properties", "nav"),
            ]),
        )?;

        match self.cover.as_ref() {
            Some(i) => {
                let file_name = i.image_file_name();
                add_element(
                    &mut xml_writer,
                    "item",
                    None,
                    Some(vec![
                        ("href", file_name.as_str()),
                        ("id", format!("i_{}", &file_name).as_str()),
                        ("media-type", i.mime_type),
                        ("properties", "cover-image"),
                    ]),
                )?;
            }
            None => {}
        };

        let mut is_first = true;
        for i in self.images.iter() {
            let img_file_name = i.image_file_name();
            let img_id = format!("i_{}", &img_file_name);
            let mut img_attrs = vec![
                ("href", img_file_name.as_str()),
                ("id", img_id.as_str()),
                ("media-type", i.mime_type),
            ];
            if is_first && self.cover.is_none() {
                img_attrs.push(("properties", "cover-image"));
            }
            is_first = false;

            add_element(&mut xml_writer, "item", None, Some(img_attrs))?;

            for j in i.page_file_names(self.metadata.right_to_left).iter() {
                add_element(
                    &mut xml_writer,
                    "item",
                    None,
                    Some(vec![
                        ("href", j.as_str()),
                        ("id", format!("p_{}", j).as_str()),
                        ("media-type", "application/xhtml+xml"),
                        ("properties", "svg"),
                    ]),
                )?;
            }
        }

        xml_writer.write(XmlEvent::end_element())?;

        xml_writer.write(XmlEvent::start_element("spine").attr(
            "page-progression-direction",
            match self.metadata.right_to_left {
                true => "rtl",
                false => "ltr",
            },
        ))?;

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
