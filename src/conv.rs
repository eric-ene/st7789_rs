use image::DynamicImage;
use itertools::izip;

pub fn bytes_from_img(img: &DynamicImage) -> Vec<u8> {
  let bytes = img.as_bytes();

  let mut r: Vec<u8> = Vec::new();
  let mut g: Vec<u8> = Vec::new();
  let mut b: Vec<u8> = Vec::new();

  let mut converted_16: Vec<[u8; 2]> = Vec::new();
  let mut converted_8: Vec<u8> = Vec::new();

  let mut counter = 1;

  for byte in bytes {
    match counter % 3 {
      0 => b.push(*byte),
      1 => r.push(*byte),
      2 => g.push(*byte),
      _ => ()
    }
    counter += 1;
  }

  for (ri, gi, bi) in izip!(r, g, b) {
    let r_565 = ((ri & 0xf8) as u16) << 8;
    let g_565 = ((gi & 0xfc) as u16) << 3;
    let b_565 = ((bi & 0xf8) as u16) >> 3;

    let rgb_565 = r_565 | g_565 | b_565;

    converted_16.push(rgb_565.to_be_bytes());
  }

  for chunk in converted_16 {
    for byte in chunk {
      converted_8.push(byte);
    }
  }

  return converted_8;
}
