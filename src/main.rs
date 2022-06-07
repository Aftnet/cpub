mod cpub_creator;

use std::fs::File;
use std::path::Path;

use crate::cpub_creator::metadata::Metadata;
use crate::cpub_creator::EpubWriter;

fn main() {
    println!("Opening");
    let mut writer = EpubWriter::new_at(Path::new("test.epub"), Metadata::default()).unwrap();
    for i in 0..4 {
        println!("Adding {}", i);
        let mut file = File::open(Path::new("test/img01.png")).unwrap();
        writer.add_image(&mut file, Option::None).unwrap();
    }
}
