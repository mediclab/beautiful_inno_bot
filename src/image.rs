use image as img;
use image::DynamicImage;
use std::path::Path;

pub struct Image {
    im: DynamicImage,
}

impl Image {
    pub fn new(path: &str) -> Self {
        Image {
            im: img::open(Path::new(&path)).unwrap(),
        }
    }

    pub fn scale(&mut self, ex: f32) -> &mut Image {
        let max_px = if self.im.width() > self.im.height() {
            (self.im.width() as f32 * ex) as u32
        } else {
            (self.im.height() as f32 * ex) as u32
        };

        self.im = self.im.thumbnail(max_px, max_px);

        self
    }

    pub fn save(&self, path: &str) -> bool {
        self.im.save(Path::new(&path)).is_ok()
    }
}
