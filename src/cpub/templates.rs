pub const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;

pub const PAGE_SPACER_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<html lang="en-US" xml:lang="en-US" xmlns="http://www.w3.org/1999/xhtml">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=IMGW, height=IMGH" />
    <title>PGTITLE</title>
  </head>
  <body>
    <svg width="IMGW" height="IMGH" viewBox="0 0 IMGW IMGH" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xml:space="preserve" />
  </body>
</html>"#;

pub const PAGE_REGULAR_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<html lang="en-US" xml:lang="en-US" xmlns="http://www.w3.org/1999/xhtml">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=IMGW, height=IMGH" />
    <title>PGTITLE</title>
  </head>
  <body>
    <svg width="IMGW" height="IMGH" viewBox="0 0 IMGW IMGH" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xml:space="preserve">
        <image x="0" y="0" width="IMGW" height="IMGH" xlink:href="FILENAME"/>
    </svg>
  </body>
</html>"#;

pub const PAGE_SPREAD_R_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<html lang="en-US" xml:lang="en-US" xmlns="http://www.w3.org/1999/xhtml">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=IMGHW, height=IMGH" />
    <title>PGTITLE</title>
  </head>
  <body>
    <svg width="IMGHW" height="IMGH" viewBox="0 0 IMGHW IMGH" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xml:space="preserve">
        <image x="-IMGHW" y="0" width="IMGW" height="IMGH" xlink:href="FILENAME"/>
    </svg>
  </body>
</html>"#;

pub const PAGE_SPREAD_L_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<html lang="en-US" xml:lang="en-US" xmlns="http://www.w3.org/1999/xhtml">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=IMGHW, height=IMGH" />
    <title>PGTITLE</title>
  </head>
  <body>
    <svg width="IMGHW" height="IMGH" viewBox="0 0 IMGHW IMGH" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xml:space="preserve">
        <image x="0" y="0" width="IMGW" height="IMGH" xlink:href="FILENAME"/>
    </svg>
  </body>
</html>"#;
