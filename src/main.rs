use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use cpub::{EpubWriter, Metadata};

fn main() {
    println!("Opening");

    let f = File::create(Path::new("test.epub")).unwrap();
    let f = BufWriter::new(f);

    let mut metadata = Metadata::default();
    //metadata.right_to_left = true;
    let mut writer = EpubWriter::new(f, metadata).unwrap();

    let mut file = File::open(Path::new("test/img01.png")).unwrap();
    writer.set_cover(&mut file).unwrap();

    for i in 0..3 {
        let mut file = File::open(Path::new("test/img01.png")).unwrap();
        writer
            .add_image(&mut file, Option::Some(format!("Bookmark {}", i)))
            .unwrap();
    }

    let mut file = File::open(Path::new("test/img02.png")).unwrap();
    writer.add_image(&mut file, Option::None).unwrap();

    writer.close().unwrap();
}
