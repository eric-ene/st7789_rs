use std::{fs::File, cmp::min, thread::sleep, time::Duration};

use color::Color;
use image::{DynamicImage, ImageBuffer, imageops::overlay};
use imageproc::{drawing::{draw_text_mut, draw_filled_rect_mut}, rect::Rect};
use rppal::gpio::{Gpio, OutputPin};
use rusttype::{Scale, Font};
use spidev::{Spidev, SpidevOptions, SpiModeFlags, SpidevTransfer};

use conv::bytes_from_img;
use text::Text;

pub mod conv;
pub mod text;
pub mod color;

struct Consts {}

#[allow(dead_code)]
impl Consts {
  const BG_SPI_CS_BACK: u8 = 0;
  const BG_SPI_CS_FRONT: u8 = 1;

  const SPI_CLOCK_HZ: u32 = 60_000_000;

  const ST7789_NOP: u8 = 0x00;
  const ST7789_SWRESET: u8 = 0x01;
  const ST7789_RDDID: u8 = 0x04;
  const ST7789_RDDST: u8 = 0x09;

  const ST7789_SLPIN: u8 = 0x10;
  const ST7789_SLPOUT: u8 = 0x11;
  const ST7789_PTLON: u8 = 0x12;
  const ST7789_NORON: u8 = 0x13;

  const ST7789_INVOFF: u8 = 0x20;
  const ST7789_INVON: u8 = 0x21;
  const ST7789_DISPOFF: u8 = 0x28;
  const ST7789_DISPON: u8 = 0x29;

  const ST7789_CASET: u8 = 0x2A;
  const ST7789_RASET: u8 = 0x2B;
  const ST7789_RAMWR: u8 = 0x2C;
  const ST7789_RAMRD: u8 = 0x2E;

  const ST7789_PTLAR: u8 = 0x30;
  const ST7789_MADCTL: u8 = 0x36;
  const ST7789_COLMOD: u8 = 0x3A;

  const ST7789_FRMCTR1: u8 = 0xB1;
  const ST7789_FRMCTR2: u8 = 0xB2;
  const ST7789_FRMCTR3: u8 = 0xB3;
  const ST7789_INVCTR: u8 = 0xB4;
  const ST7789_DISSET5: u8 = 0xB6;

  const ST7789_GCTRL: u8 = 0xB7;
  const ST7789_GTADJ: u8 = 0xB8;
  const ST7789_VCOMS: u8 = 0xBB;

  const ST7789_LCMCTRL: u8 = 0xC0;
  const ST7789_IDSET: u8 = 0xC1;
  const ST7789_VDVVRHEN: u8 = 0xC2;
  const ST7789_VRHS: u8 = 0xC3;
  const ST7789_VDVS: u8 = 0xC4;
  const ST7789_VMCTR1: u8 = 0xC5;
  const ST7789_FRCTRL2: u8 = 0xC6;
  const ST7789_CABCCTRL: u8 = 0xC7;

  const ST7789_RDID1: u8 = 0xDA;
  const ST7789_RDID2: u8 = 0xDB;
  const ST7789_RDID3: u8 = 0xDC;
  const ST7789_RDID4: u8 = 0xDD;

  const ST7789_GMCTRP1: u8 = 0xE0;
  const ST7789_GMCTRN1: u8 = 0xE1;

  const ST7789_PWCTR6: u8 = 0xFC;
}


pub enum DataType {
  Data,
  Command
}

#[allow(dead_code)]
pub struct ST7789 {
  gpio: Gpio,
  spi: Spidev,

  cs: OutputPin,
  dc: OutputPin,
  bl: OutputPin,
  rst: Option<OutputPin>, // optional

  width: i16, // dimensions
  height: i16, // ^
  rotation: f32, // optional w/ default (0)
  offset_x: i16, // optional w/ default (0)
  offset_y: i16, // optional w/ default (0)

  invert: bool,

  display_buffer: DynamicImage,

  chunk_size: Option<usize>
}

impl ST7789 {
  pub fn new(
    spi_no: u8, 
    dev_no: u8,
    cs_no: u8,
    dc_no: u8,
    bl_no: u8,
    speed_hz: u32
  ) -> Self {
    let mut spi = Spidev::new(File::open(format!("/dev/spidev{}.{}", spi_no, dev_no)).expect("COULDN'T OPEN SPI FILE!"));
    let gpio = Gpio::new().unwrap();
    
    // pins
    let cs = gpio.get(cs_no).unwrap().into_output();
    let dc = gpio.get(dc_no).unwrap().into_output();
    let bl = gpio.get(bl_no).unwrap().into_output();

    // options
    let opts = SpidevOptions::new()
      .bits_per_word(0)
      .max_speed_hz(speed_hz)
      .lsb_first(false)
      .mode(SpiModeFlags::SPI_MODE_0)
      .build();

    spi.configure(&opts).expect(
      "Couldn't configure SPI! \
      \nThis is likely because SPI is not set\
      \nup on your pi."
    );
  
    return Self {
      gpio: gpio,
      spi: spi,
      //pwm: pwm,

      cs: cs,
      dc: dc,
      bl: bl,
      rst: None,

      width: 320,
      height: 170,
      rotation: 0_f32,
      offset_x: 0,
      offset_y: 0,

      invert: true,
      display_buffer: DynamicImage::ImageRgb8(ImageBuffer::new(320, 170)),

      chunk_size: None
    }
  }

  pub fn draw_image(&mut self, img: &DynamicImage, pos_x: i16, pos_y: i16) {
    overlay(&mut self.display_buffer, img, pos_x as i64, pos_y as i64);
  }

  pub fn draw_text(&mut self, text: &str, font: &Font, pos_x: i16, pos_y: i16, color: &Color, size: f32) {
    draw_text_mut(
      &mut self.display_buffer, 
      color.get_rgba(), 
      pos_x as i32, pos_y as i32, 
      Scale::uniform(size), 
      font, 
      text
    );
  }

  pub fn draw_text_obj(&mut self, text: &Text, pos_x: i16, pos_y: i16) {
   self.draw_text( 
      text.content.as_str(),
      &text.font,
      pos_x, pos_y,
      &text.color,
      text.size
    );
  }

  pub fn draw_rect(&mut self, rect: Rect, color: &Color) {
    draw_filled_rect_mut(
      &mut self.display_buffer,
      rect,
      color.get_rgba()
    );
  }

  pub fn draw_clear(&mut self, color: &Color) {
    draw_filled_rect_mut(
      &mut self.display_buffer,
      Rect::at(0, 0)
        .of_size(self.width as u32, self.height as u32), 
      color.get_rgba()
    );
  }

  pub fn display(&mut self) {
    self.set_window(None, None, None, None);

    let bytes_vec = bytes_from_img(&self.display_buffer);
    let bytes = bytes_vec.as_slice();

    self.send_cmd(Consts::ST7789_RAMWR);
    self.send_datas(bytes);
  }

  pub fn set_window(&mut self, x1_o: Option<i16>, y1_o: Option<i16>, x2_o: Option<i16>, y2_o: Option<i16>) {
    let mut x1 = match x1_o { Some(x) => x, None => 0 };
    let mut y1 = match y1_o { Some(y) => y, None => 0 };
    let mut x2 = match x2_o { Some(x) => x-1, None => self.width-1 };
    let mut y2 = match y2_o { Some(y) => y-1, None => self.height-1 };

    x1 += self.offset_x;
    y1 += self.offset_y;
    x2 += self.offset_x;
    y2 += self.offset_y;
 
    // column addr set
    self.send_cmd(Consts::ST7789_CASET);
    self.send_data((x1 >> 8) as u8); // x start
    self.send_data((x1 & 0xff) as u8); // ^ other half
    self.send_data((x2 >> 8) as u8); // x end
    self.send_data((x2 & 0xff) as u8); // ^ other half

    // row addr set
    self.send_cmd(Consts::ST7789_RASET);
    self.send_data((y1 >> 8) as u8); // x start
    self.send_data((y1 & 0xff) as u8); // ^ other half
    self.send_data((y2 >> 8) as u8); // x end
    self.send_data((y2 & 0xff) as u8); // ^ other half

  }

  pub fn clear(&mut self, col: u16) {
    let color_half = col.to_be_bytes();
    let num_pixels = self.width as i32 * self.height as i32;
    let mut pix_vec: Vec<u8> = Vec::new();

    self.set_window(None, None, None, None);

    for _ in 0..num_pixels {
      pix_vec.push(color_half[0]);
      pix_vec.push(color_half[1]);
    }

    self.send_cmd(Consts::ST7789_RAMWR);
    self.send_datas(pix_vec.as_slice());
  }

  // TODO: Doesn't work as of now
  pub fn reset(&mut self) {
    match &mut self.rst {
      Some(p) => {
        p.set_high();
        sleep(Duration::from_millis(10));
        p.set_low();
        sleep(Duration::from_millis(10));
        p.set_high();
        sleep(Duration::from_millis(10));
      }
      None => {
        println!("Reset pin not set.");
      }
    }
  }

  pub fn init(&mut self) {
    self.bl.set_pwm_frequency(1000.0, 1.0).expect(
      "Couldn't set PWM frequency! \
      \nThis is likely because your PWM outputs \
      \nare disabled."
    );
    self.reset();

    self.send_cmd(Consts::ST7789_SWRESET);
    sleep(Duration::from_millis(150));

    // mem access data ctrl
    // format: <bit> - <description> - <0>/<1>
    // from least to most significant:
    // 0, 1: - ignored
    // 2 - LCD refresh     - l-r / r-l
    // 3 - color           - rgb / bgr
    // 4 - line addr order - t-b / b-t
    // 5 - page/col order  - normal / reversed
    // 6 - col addr order  - l-r / r-l
    // 7 - page addr order - t-b / b-t
  
    self.send_cmd(Consts::ST7789_MADCTL);
    self.send_data(0x70);

    self.send_cmd(Consts::ST7789_FRMCTR2); // frame ctrl, normal mode
    self.send_data(0x0C);
    self.send_data(0x0C);
    self.send_data(0x00);
    self.send_data(0x33);
    self.send_data(0x33);

    self.send_cmd(Consts::ST7789_COLMOD); // color mode
    self.send_data(0x05); // rgb565

    self.send_cmd(Consts::ST7789_GCTRL); // gate ctrl
    self.send_data(0x35);

    self.send_cmd(Consts::ST7789_VCOMS); 
    self.send_data(0x13);

    self.send_cmd(Consts::ST7789_LCMCTRL); // power ctrl
    self.send_data(0x2C);

    self.send_cmd(Consts::ST7789_VDVVRHEN); // power ctrl
    self.send_data(0x01);

    self.send_cmd(Consts::ST7789_VRHS); // power ctrl
    self.send_data(0x0B);

    self.send_cmd(Consts::ST7789_VDVS); // power ctrl
    self.send_data(0x20);

    self.send_cmd(0xd0); // power ctrl
    self.send_data(0xa4);
    self.send_data(0xa1);

    self.send_cmd(Consts::ST7789_FRCTRL2); // framerate ctrl
    self.send_data(0x0f);

    self.send_cmd(Consts::ST7789_GMCTRP1); // gamma +
    self.send_data(0x00);
    self.send_data(0x03);
    self.send_data(0x07);
    self.send_data(0x08);
    self.send_data(0x07);
    self.send_data(0x15);
    self.send_data(0x2A);
    self.send_data(0x44);
    self.send_data(0x42);
    self.send_data(0x0A);
    self.send_data(0x17);
    self.send_data(0x18);
    self.send_data(0x25);
    self.send_data(0x27);

    self.send_cmd(Consts::ST7789_GMCTRN1); // gamma -
    self.send_data(0x00);
    self.send_data(0x03);
    self.send_data(0x08);
    self.send_data(0x07);
    self.send_data(0x07);
    self.send_data(0x23);
    self.send_data(0x2A);
    self.send_data(0x43);
    self.send_data(0x42);
    self.send_data(0x09);
    self.send_data(0x18);
    self.send_data(0x17);
    self.send_data(0x25);
    self.send_data(0x27);

    self.send_cmd(match self.invert {
        true => Consts::ST7789_INVON,
        false => Consts::ST7789_INVOFF
    });

    self.send_cmd(Consts::ST7789_SLPOUT);

    self.send_cmd(Consts::ST7789_DISPON);
  }

  pub fn cleanup(&mut self) {
    match &mut self.rst {
      Some(pin) => {
        pin.set_high();
        self.dc.set_low();
        sleep(Duration::from_millis(1));
        self.bl.set_high();
      },
      None => return
    }
  }

  pub fn send_data(&mut self, data: u8) {
    self.send_datas(&[data]);
  }

  pub fn send_cmd(&mut self, data: u8) {
    self.send_cmds(&[data]);
  }

  pub fn send_datas(&mut self, data: &[u8]) {
    self.send(data, DataType::Data);
  }

  pub fn send_cmds(&mut self, data: &[u8]) {
    self.send(data, DataType::Command);
  }

  pub fn send(&mut self, data: &[u8], data_type: DataType) {
    // set data mode (high for data, low for command)
    match data_type {
      DataType::Command => self.dc.set_low(),
      DataType::Data => self.dc.set_high()
    };

    // get chunk size
    let step = match self.chunk_size {
      Some(s) => s,
      None => 4096
    };

    // send the data <step> bytes at a time
    for start in (0..data.len()).step_by(step) {
      let end = min(start + step, data.len());
      let mut slice = SpidevTransfer::write(&data[start..end]);

      match self.spi.transfer(&mut slice) {
        Ok(_) => (),
        Err(_) => println!("Error sending command/data starting with {:#04x}", data[start]),
      }
    }
  }

  pub fn with_reset(mut self, rst_no: u8) -> Self {
    self.rst =Some(self.gpio.get(rst_no).unwrap().into_output());

    return self;
  }

  pub fn with_dimensions(mut self, width: i16, height: i16) -> Self {
    self.width = width;
    self.height = height;

    return self;
  }

  pub fn with_rotation(mut self, rot: f32) -> Self {
    self.rotation = rot;

    return self;
  }

  pub fn with_offset(mut self, off_x: i16, off_y: i16) -> Self {
    self.offset_x = off_x;
    self.offset_y = off_y;

    return self;
  }

  pub fn with_chunk_size(mut self, size: usize) -> Self {
    self.chunk_size = Some(size);

    return self;
  }

  pub fn width(&self) -> i16 {
    return self.width;
  }

  pub fn height(&self) -> i16 {
    return self.height;
  }
}