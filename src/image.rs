use image as img;
use image::DynamicImage;
use std::path::Path;

pub struct Image {
    im: DynamicImage,
}

impl Image {
    pub fn new(path: &Path) -> Self {
        Image {
            im: img::open(path).unwrap(),
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

    pub fn resize(&mut self, px: u32) -> &mut Image {
        self.im = self.im.thumbnail(px, px);

        self
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.im.width(), self.im.height())
    }

    pub fn get_scaling(&self, max_size: u32) -> f32 {
        let (width, height) = self.get_size();
        let scale_y = max_size as f32 / width as f32;
        let scale_x = max_size as f32 / height as f32;

        if scale_y > scale_x {
            scale_x
        } else {
            scale_y
        }
    }

    pub fn save(&self, path: &Path) -> bool {
        self.im.save(path).is_ok()
    }
}
