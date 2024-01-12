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
        if let Some(field) = self.get_field_string(&Tag::Make) {
            return field;
        }

        String::new()
    }

    pub fn get_model(&self) -> String {
        if let Some(field) = self.get_field_string(&Tag::Model) {
            return field;
        }

        String::new()
    }

    pub fn get_exposure_time(&self) -> String {
        if let Some(field) = self.get_field_string(&Tag::ExposureTime) {
            return format!("{}s", field);
        }

        String::new()
    }

    pub fn get_focal_number(&self) -> String {
        if let Some(field) = self.get_field_string(&Tag::FNumber) {
            return format!("f/{}", field);
        }

        String::new()
    }

    pub fn get_focal_length(&self) -> String {
        if let Some(field) = self.get_field_string(&Tag::FocalLength) {
            return format!("{}mm", field);
        }

        String::new()
    }

    pub fn get_iso(&self) -> String {
        if let Some(field) = self.get_field_string(&Tag::PhotographicSensitivity) {
            return format!("ISO{}", field);
        }

        String::new()
    }

    pub fn get_photo_info_string(&self) -> String {
        format!(
            "{} {} {} {}",
            self.get_focal_number(),
            self.get_exposure_time(),
            self.get_focal_length(),
            self.get_iso()
        )
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
    }

    fn get_field_string(&self, tag: &Tag) -> Option<String> {
        if let Some(field) = self.exif.get_field(*tag, In::PRIMARY) {
            debug!("{} field: {:?}", field.tag, field.value);
            return match field.value {
                Value::Rational(ref v) if !v.is_empty() => Some(field.display_value().to_string()),
                Value::Ascii(ref v) if !v.is_empty() => {
                    Some(field.display_value().to_string().replace('"', ""))
                }
                Value::Short(ref v) if !v.is_empty() => Some(field.display_value().to_string()),
                _ => None,
            };
        }

        None
    }
}
