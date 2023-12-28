use image::{Rgb, Rgba};

pub struct Color {
  hex: u32,
  rgb565: u16,
}

impl Color {
  pub fn new(hex: u32) -> Self {
    let mut r: u16 = (hex >> 16) as u16;
    let mut g: u16 = (hex >> 8 & 0xff) as u16;
    let mut b: u16 = (hex & 0xff) as u16;

    r = r & 0xf8 << 8;
    g = g & 0xfc << 3;
    b = b & 0xf8 >> 3;

    return Self {
      hex: hex,
      rgb565: r | g | b
    }
  }

  pub fn get_rgb565(&self) -> u16 {
    return self.rgb565;
  }

  pub fn get_rgb(&self) -> Rgb<u8> {
    let r = (self.hex >> 16) as u8;
    let g = (self.hex >> 8 & 0xff) as u8;
    let b = (self.hex & 0xff) as u8;

    return Rgb([r, g, b]);
  }

  pub fn get_rgba(&self) -> Rgba<u8> {
    let r = (self.hex >> 16) as u8;
    let g = (self.hex >> 8 & 0xff) as u8;
    let b = (self.hex & 0xff) as u8;

    return Rgba([r, g, b, 0xff]);
  }
}