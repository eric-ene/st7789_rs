# st7789-rs

## Information
### What is this?
- A driver
  - Primarily for the Raspberry Pi Zero 2
  - For TFT ST7789 displays
- A pet project
  - I'm mainly making this for fun, but I thought I'd publish it just in case it'd be useful for anyone else
- A port (kind of)
  - Although many elements of this package are my own, I have used [this library](https://github.com/pimoroni/st7789-python) as a reference for some of the trickier stuff (mainly the setup commands)

### What is this *not*?
- Well-documented
  - Since this is more of a personal thing, documentation isn't a very high priority at the moment. I will, however, add documentation and remove this bullet point eventually
- All-encompassing
  - This is not made or tested for any arbitrary Pi-like computer/microcontroller
  - This is not tested on every type of ST7789 display
    - Currently only tested on a Waveshare 1.9" LCD
- Official
  - I am not affiliated in any way shape or form with Waveshare or the Raspberry Pi Foundation

### Usage
```rust
  // create and init display device
  let mut device = ST7789::new(
    0,
    0,
    CS,
    DC,
    BL,
    60_000_000
  )
  .with_reset(RST)
  .with_offset(OFF_X, OFF_Y) // optional
  .with_dimensions(WIDTH, HEIGHT) // optional
  .with_rotation(90.0); // optional, currently does nothing.

  // initialize the device
  device.init();

  // now go crazy
```