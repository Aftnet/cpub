use super::{errors::EpubWriterError, templates};
use image::{GenericImageView, ImageFormat};

pub struct PageImage {
    pub base_name: String,
    pub nav_label: Option<String>,
    pub extension: &'static str,
    pub mime_type: &'static str,
    pub size: (u32, u32),
    pub spread: bool,
}

impl PageImage {
    pub fn new(image_data: &[u8], nav_label: Option<String>) -> Result<PageImage, EpubWriterError> {
        let imgfmt = image::guess_format(&image_data)
            .map_err(|source| EpubWriterError::InvalidImageError(source))?;
        let imgtypeinfo = match imgfmt {
            ImageFormat::Gif => (".gif", "image/gif"),
            ImageFormat::Jpeg => (".jpg", "image/jpeg"),
            ImageFormat::Png => (".png", "image/png"),
            _ => return Err(EpubWriterError::UnsupportedImageError),
        };

        let img = image::load_from_memory(&image_data)
            .map_err(|source| EpubWriterError::InvalidImageError(source))?;
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

    pub fn page_file_names(&self, reading_rtl: bool) -> Vec<String> {
        if self.spread {
            let mut output = vec![
                self.page_spread_left_file_name(),
                self.page_spread_right_file_name(),
            ];
            if reading_rtl {
                output.reverse();
            }
            return output;
        } else {
            vec![self.page_regular_file_name()]
        }
    }

    pub fn generate_pages_xml(&self, reading_rtl: bool) -> Vec<(String, String)> {
        if self.spread {
            let mut output = vec![
                (
                    self.page_spread_left_file_name(),
                    self.generate_page_xml(templates::PAGE_SPREAD_L_XML),
                ),
                (
                    self.page_spread_right_file_name(),
                    self.generate_page_xml(templates::PAGE_SPREAD_R_XML),
                ),
            ];
            if reading_rtl {
                output.reverse();
            }
            return output;
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
        format!("{}_R.xhtml", self.base_name)
    }

    fn generate_page_xml(&self, template: &str) -> String {
        template
            .replace("IMGW", &format!("{}", &self.size.0))
            .replace("IMGHW", &format!("{}", &self.size.0 / 2))
            .replace("IMGH", &format!("{}", &self.size.1))
            .replace("FILENAME", &self.image_file_name())
    }
}
