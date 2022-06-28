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
    cover_spacer_required: bool,
    spread_added: bool,
    finalized: bool,
    current_chapter_number: u32,
    current_page_number: u32,
    total_pages_number: u32,
    inner: ZipWriter<W>,
}

impl<W: Write + Seek> EpubWriter<W> {
    pub fn new(inner: W, metadata: Metadata) -> Result<EpubWriter<W>, EpubWriterError> {
        metadata.validate()?;

        let mut output = EpubWriter {
            metadata: metadata,
            images: Vec::default(),
            cover: None,
            cover_spacer_required: false,
            spread_added: false,
            finalized: false,
            current_chapter_number: 0,
            current_page_number: 0,
            total_pages_number: 0,
            inner: zip::ZipWriter::new(inner),
        };

        output.add_static_data()?;
        return Ok(output);
    }

    pub fn set_cover<T: std::io::Read>(&mut self, image: &mut T) -> Result<(), EpubWriterError> {
        if self.finalized {
            return Err(EpubWriterError::FinalizedError());
        }

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
        let pages = page_image.generate_pages_xml(self.metadata.right_to_left);

        self.add_zip_entry(&format!("OEBPS/{}", &img_filename), &buffer)?;
        for i in pages.iter() {
            self.add_zip_entry(&format!("OEBPS/{}", &i.0), &i.1.as_bytes())?;
        }

        return Ok(());
    }

    pub fn add_image<T: std::io::Read>(
        &mut self,
        image: &mut T,
        label: Option<String>,
    ) -> Result<(), EpubWriterError> {
        if self.finalized {
            return Err(EpubWriterError::FinalizedError());
        }

        let mut buffer: Vec<u8> = Vec::new();
        image.read_to_end(&mut buffer)?;
        let mut page_image = PageImage::new(&buffer, label)?;
        if page_image.spread {
            if self.total_pages_number % 2 == 0 {
                self.total_pages_number += 2;
            } else {
                if self.spread_added == false {
                    self.cover_spacer_required = true;
                    self.total_pages_number += 1;
                } else {
                    return Err(errors::EpubWriterError::PageSortingError {
                        page_number: (self.images.len() + 1) as u32,
                    });
                }
            }
            self.spread_added = true;
        } else {
            self.total_pages_number += 1;
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

    pub fn finalize(&mut self) -> Result<(), EpubWriterError> {
        if self.finalized {
            return Ok(());
        }

        self.finalized = true;
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
        if self.cover.is_none() {
            return Err(EpubWriterError::CoverNotSetError);
        }

        if self.images.is_empty() {
            return Err(EpubWriterError::NoPagesError);
        }

        let xml = self.generate_content_opf()?;
        self.add_zip_entry("OEBPS/content.opf", &xml)?;

        let xml = self.generate_nav_xml()?;
        self.add_zip_entry("OEBPS/nav.xhtml", &xml)?;

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
            if let Some(d) = attributes {
                for i in d.iter() {
                    xml_event = xml_event.attr(i.0, i.1);
                }
            }

            writer.write(xml_event)?;

            if let Some(d) = content {
                writer.write(XmlEvent::characters(d))?;
            }

            writer.write(XmlEvent::end_element())?;
            Ok(())
        }

        fn manifest_add_image<W: Write>(
            xml_writer: &mut EventWriter<W>,
            base_name: &str,
            file_name: &str,
            mime_type: &str,
            is_cover: bool,
        ) -> xml::writer::Result<()> {
            let id = format!("i_{}", base_name);
            let mut attrs = vec![
                ("href", file_name),
                ("id", id.as_str()),
                ("media-type", mime_type),
            ];
            if is_cover {
                attrs.push(("properties", "cover-image"));
            }

            return add_element(xml_writer, "item", None, Some(attrs));
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
            Some(self.metadata.id.as_str()),
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
            Some(self.metadata.title.as_str()),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:creator",
            Some(self.metadata.author.as_str()),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:publisher",
            Some(self.metadata.publisher.as_str()),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:date",
            Some(
                self.metadata
                    .published_date
                    .format("%Y-%m-%d")
                    .to_string()
                    .as_str(),
            ),
            None,
        )?;
        add_element(
            &mut xml_writer,
            "dc:language",
            Some(self.metadata.language.as_str()),
            None,
        )?;
        if let Some(d) = self.metadata.description.as_ref() {
            add_element(&mut xml_writer, "dc:description", Some(d), None)?;
        }

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

        if let Some(d) = self.metadata.source.as_ref() {
            add_element(&mut xml_writer, "dc:source", Some(d), None)?;
        }
        if let Some(d) = self.metadata.relation.as_ref() {
            add_element(&mut xml_writer, "dc:relation", Some(d), None)?;
        }
        if let Some(d) = self.metadata.copyright.as_ref() {
            add_element(&mut xml_writer, "dc:rights", Some(d), None)?;
        }

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

        let cover = self.cover.as_ref().unwrap();
        manifest_add_image(
            &mut xml_writer,
            cover.base_name.as_str(),
            cover.cover_file_name().as_str(),
            cover.mime_type,
            true,
        )?;

        let page_file_name = cover.cover_file_name();
        add_element(
            &mut xml_writer,
            "item",
            None,
            Some(vec![
                ("href", page_file_name.as_str()),
                ("id", format!("p_{}", page_file_name).as_str()),
                ("media-type", "application/xhtml+xml"),
                ("properties", "svg"),
            ]),
        )?;

        for i in self.images.iter() {
            manifest_add_image(
                &mut xml_writer,
                i.base_name.as_str(),
                i.image_file_name().as_str(),
                i.mime_type,
                false,
            )?;

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

        if let Some(ref cover) = self.cover {
            let idref = format!("p_{}", cover.cover_file_name());
            let attrs = vec![("idref", idref.as_str()), ("linear", "no")];

            add_element(&mut xml_writer, "itemref", None, Some(attrs))?;
        }

        for i in self.images.iter() {
            for j in i.page_file_names(self.metadata.right_to_left).iter() {
                let idref = format!("p_{}", j);
                let mut attrs = vec![("idref", idref.as_str())];
                if j.ends_with("_L.xhtml") {
                    attrs.push(("properties", "page-spread-left"));
                } else if j.ends_with("_R.xhtml") {
                    attrs.push(("properties", "page-spread-right"));
                }

                add_element(&mut xml_writer, "itemref", None, Some(attrs))?;
            }
        }

        xml_writer.write(XmlEvent::end_element())?;

        xml_writer.write(XmlEvent::end_element())?;
        return Ok(buffer);
    }

    fn generate_nav_xml(&mut self) -> xml::writer::Result<Vec<u8>> {
        let mut buffer = Vec::<u8>::new();
        let mut xml_writer = EventWriter::new_with_config(
            Cursor::new(&mut buffer),
            EmitterConfig {
                perform_indent: true,
                ..Default::default()
            },
        );

        xml_writer.write(
            XmlEvent::start_element("html")
                .default_ns("http://www.w3.org/1999/xhtml")
                .ns("epub", "http://www.idpf.org/2007/ops"),
        )?;
        xml_writer.write(XmlEvent::start_element("head"))?;
        xml_writer.write(XmlEvent::start_element("meta").attr("charset", "utf-8"))?;
        xml_writer.write(XmlEvent::end_element())?;
        xml_writer.write(XmlEvent::start_element("title"))?;
        xml_writer.write(XmlEvent::characters("Navigation"))?;
        xml_writer.write(XmlEvent::end_element())?;
        xml_writer.write(XmlEvent::end_element())?;

        xml_writer.write(XmlEvent::start_element("body"))?;
        xml_writer.write(
            XmlEvent::start_element("nav")
                .attr("id", "nav")
                .attr("epub:type", "toc"),
        )?;
        xml_writer.write(XmlEvent::start_element("ol"))?;

        let bookmarks: Vec<_> = self
            .images
            .iter()
            .filter(|&d| d.nav_label.is_some())
            .collect();
        if bookmarks.is_empty() {
            xml_writer.write(XmlEvent::start_element("li"))?;
            xml_writer.write(
                XmlEvent::start_element("a")
                    .attr("href", self.cover.as_ref().unwrap().cover_file_name().as_str()),
            )?;
            xml_writer.write(XmlEvent::characters(self.metadata.title.as_str()))?;
            xml_writer.write(XmlEvent::end_element())?;
            xml_writer.write(XmlEvent::end_element())?;
        } else {
            for i in bookmarks.into_iter() {
                xml_writer.write(XmlEvent::start_element("li"))?;
                xml_writer.write(
                    XmlEvent::start_element("a").attr(
                        "href",
                        i.page_file_names(self.metadata.right_to_left)
                            .first()
                            .unwrap()
                            .as_str(),
                    ),
                )?;
                xml_writer.write(XmlEvent::characters(i.nav_label.as_ref().unwrap().as_str()))?;
                xml_writer.write(XmlEvent::end_element())?;
                xml_writer.write(XmlEvent::end_element())?;
            }
        }

        xml_writer.write(XmlEvent::end_element())?;
        xml_writer.write(XmlEvent::end_element())?;
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
