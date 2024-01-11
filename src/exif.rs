use exif::{Exif, In, Reader, Tag, Value};
use std::io::BufReader;

pub struct ExifLoader {
    exif: Exif,
}

impl ExifLoader {
    pub fn new(file_path: String) -> Self {
        let file = std::fs::File::open(file_path).expect("I/O Error");
        let mut bufreader = BufReader::new(&file);
        let exif = Reader::new()
            .read_from_container(&mut bufreader)
            .expect("Can't load Exif info");

        ExifLoader { exif }
    }

    pub fn get_maker(&self) -> String {
        if let Some(make_field) = self.exif.get_field(Tag::Make, In::PRIMARY) {
            return match make_field.value {
                Value::Ascii(ref v) if !v.is_empty() => {
                    make_field.display_value().to_string().replace('"', "")
                }
                _ => String::new(),
            };
        }

        String::new()
    }

    pub fn get_model(&self) -> String {
        if let Some(model_field) = self.exif.get_field(Tag::Model, In::PRIMARY) {
            return match model_field.value {
                Value::Ascii(ref v) if !v.is_empty() => {
                    model_field.display_value().to_string().replace('"', "")
                }
                _ => String::new(),
            };
        }

        String::new()
    }
}
