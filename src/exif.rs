use anyhow::{Error, bail};
use exif::{Exif, In, Reader, Tag, Value};
use inflector::Inflector;
use std::{io::BufReader, path::Path};

pub struct ExifLoader {
    exif: Exif,
}

impl ExifLoader {
    pub fn new(file_path: &Path) -> Result<Self, Error> {
        let file = std::fs::File::open(file_path).expect("I/O Error");
        let mut bufreader = BufReader::new(&file);

        match Reader::new().read_from_container(&mut bufreader) {
            Ok(exif) => Ok(ExifLoader { exif }),
            Err(_) => bail!("Can't get Exif information!"),
        }
    }

    pub fn get_maker(&self) -> Option<String> {
        if let Some(field) = self.get_field_string(&Tag::Make) {
            return Some(field);
        }

        None
    }

    pub fn get_model(&self) -> Option<String> {
        if let Some(field) = self.get_field_string(&Tag::Model) {
            return Some(field);
        }

        None
    }

    // pub fn get_lens_model(&self) -> Option<String> {
    //     if let Some(field) = self.get_field_string(&Tag::LensModel) {
    //         return Some(field);
    //     }

    //     None
    // }

    pub fn get_software(&self) -> Option<String> {
        if let Some(field) = self.get_field_string(&Tag::Software) {
            return Some(field);
        }

        None
    }

    pub fn get_exposure_time(&self) -> Option<String> {
        if let Some(field) = self.get_field_string(&Tag::ExposureTime) {
            return Some(format!("{field}s"));
        }

        None
    }

    pub fn get_focal_number(&self) -> Option<String> {
        if let Some(field) = self.get_field_string(&Tag::FNumber) {
            return Some(format!("f/{field}"));
        }

        None
    }

    pub fn get_focal_length(&self) -> Option<String> {
        if let Some(field) = self.get_field_string(&Tag::FocalLength) {
            return Some(format!("{field}mm"));
        }

        None
    }

    pub fn get_iso(&self) -> Option<String> {
        if let Some(field) = self.get_field_string(&Tag::PhotographicSensitivity) {
            return Some(format!("ISO{field}"));
        }

        None
    }

    pub fn get_photo_info_string(&self) -> Option<String> {
        let infos = vec![self.get_focal_number(), self.get_exposure_time(), self.get_focal_length(), self.get_iso()]
            .into_iter()
            .flatten()
            .collect::<Vec<String>>();

        if !infos.is_empty() {
            return Some(infos.join(" "));
        }

        None
    }

    pub fn get_maker_model(&self) -> Option<String> {
        let o_maker = self.get_maker();
        let o_model = self.get_model();

        if o_maker.is_none() && o_model.is_none() {
            return None;
        } else if o_maker.is_none() {
            return Some(o_model.unwrap_or_default());
        } else if o_model.is_none() {
            return Some(o_maker.unwrap_or_default().to_title_case());
        }

        let model = o_model.unwrap_or_default();
        let maker = o_maker.unwrap_or_default();

        if model.to_ascii_lowercase().contains(&maker.to_ascii_lowercase()) {
            Some(model.to_string())
        } else {
            Some(format!("{} {}", maker.to_title_case(), model))
        }
    }

    fn get_field_string(&self, tag: &Tag) -> Option<String> {
        if let Some(field) = self.exif.get_field(*tag, In::PRIMARY) {
            debug!("{} field: {:?}", field.tag, field.value);
            return match field.value {
                Value::Rational(ref v) if !v.is_empty() => {
                    let denom = v[0].denom as f64;
                    let num = v[0].num as f64;

                    let res = match field.tag {
                        Tag::ExposureTime => format!("1/{:.0}", denom / num),
                        Tag::FNumber => format!("{:.2}", num / denom),
                        Tag::FocalLength => format!("{:.2}", num / denom),
                        _ => field.display_value().to_string(),
                    };

                    Some(res)
                }
                Value::Ascii(ref v) if !v.is_empty() => {
                    if let Ok(val) = String::from_utf8(v[0].clone()) {
                        Some(val.replace('"', ""))
                    } else {
                        None
                    }
                }
                Value::Short(ref v) if !v.is_empty() => Some(field.display_value().to_string()),
                _ => None,
            };
        }

        None
    }
}
