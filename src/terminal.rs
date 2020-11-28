use embedded_graphics as eg;

use eg::fonts::{Font6x12, Text};
use eg::pixelcolor::Rgb565;
use eg::prelude::*;
use eg::primitives::rectangle::Rectangle;
use eg::style::{PrimitiveStyleBuilder, TextStyle};

use wio_terminal::{Scroller, LCD};

use cortex_m::asm::delay as cycle_delay;

type TextSegment = ([u8; 32], usize);

// From https://github.com/atsamd-rs/atsamd/blob/4ddb4dbc75c509f3b5044c540fcb7b0452543712/boards/wio_terminal/examples/usb_serial_display.rs#L115
// By @jessebraham
pub struct Terminal {
    text_style: TextStyle<Rgb565, Font6x12>,
    cursor: Point,
    display: LCD,
    scroller: Scroller,
}

impl Terminal {
    pub fn new(mut display: LCD) -> Self {
        // Clear the screen.
        let style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLACK)
            .build();
        let backdrop = Rectangle::new(Point::new(0, 0), Point::new(320, 320)).into_styled(style);
        backdrop.draw(&mut display).ok().unwrap();

        let scroller = display.configure_vertical_scroll(0, 0).unwrap();

        Self {
            text_style: TextStyle::new(Font6x12, Rgb565::WHITE),
            cursor: Point::new(0, 0),
            display,
            scroller,
        }
    }

    pub fn write_str(&mut self, str: &str) {
        for character in str.chars() {
            self.write_character(character);
        }
    }

    pub fn write_character(&mut self, c: char) {
        if self.cursor.x >= 320 || c == '\n' {
            self.cursor = Point::new(0, self.cursor.y + Font6x12::CHARACTER_SIZE.height as i32);
        }
        if self.cursor.y >= 240 {
            self.animate_clear();
            self.cursor = Point::new(0, 0);
        }

        if c != '\n' {
            let mut buf = [0u8; 8];
            Text::new(c.encode_utf8(&mut buf), self.cursor)
                .into_styled(self.text_style)
                .draw(&mut self.display)
                .ok()
                .unwrap();

            self.cursor.x += (Font6x12::CHARACTER_SIZE.width + Font6x12::CHARACTER_SPACING) as i32;
        }
    }

    pub fn write(&mut self, segment: TextSegment) {
        let (buf, count) = segment;
        for (i, character) in buf.iter().enumerate() {
            if i >= count {
                break;
            }
            self.write_character(*character as char);
        }
    }

    fn animate_clear(&mut self) {
        for x in (0..320).step_by(Font6x12::CHARACTER_SIZE.width as usize) {
            self.display
                .scroll_vertically(&mut self.scroller, Font6x12::CHARACTER_SIZE.width as u16)
                .ok()
                .unwrap();
            Rectangle::new(
                Point::new(x + 0, 0),
                Point::new(x + Font6x12::CHARACTER_SIZE.width as i32, 240),
            )
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::BLACK)
                    .build(),
            )
            .draw(&mut self.display)
            .ok()
            .unwrap();

            cycle_delay(1000);
        }
    }
}
