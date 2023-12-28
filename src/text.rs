use rusttype::{Font, Scale, point};

use crate::color::Color;

pub struct Text<'a> {
  pub content: &'a str,
  pub size: f32,
  pub font: Font<'a>,
  pub color: Color
}

impl<'a> Text<'a> {
  pub fn new(text: &'a str, size: f32, font: Font<'a>, color: Color) -> Self {
    Self {
      content: text,
      size: size,
      font: font,
      color: color
    }
  }

  pub fn get_width(&self) -> u32 {
    let glyphs: Vec<_> = self.font.layout(self.content, Scale::uniform(self.size), point(0.0, 0.0)).collect();
    let glyphs_width = {
      let min_x = glyphs
          .first()
          .map(|g| g.pixel_bounding_box().unwrap().min.x)
          .unwrap();
      let max_x = glyphs
          .last()
          .map(|g| g.pixel_bounding_box().unwrap().max.x)
          .unwrap();
      (max_x - min_x) as u32
    };

    return glyphs_width;
  } 
}