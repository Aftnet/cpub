use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use cpub::{EpubWriter, Metadata};

fn main() {
    println!("Opening");

    let f = File::create(Path::new("test.epub")).unwrap();
    let f = BufWriter::new(f);
    let mut writer = EpubWriter::new(f, Metadata::default()).unwrap();

    for i in 0..4 {
        println!("Adding {}", i);
        let mut file = File::open(Path::new("test/img01.png")).unwrap();
        writer.add_image(&mut file, Option::None).unwrap();
    }
}
