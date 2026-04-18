use core::fmt::Write;

use embedded_graphics::mono_font::jis_x0201::FONT_10X20;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use heapless::String;

use crate::init::Lcd;

/// A const function to dynamically create the coords for the 4 axes.
/// Maybe a bit overkill.
const fn axes_coords() -> [(i32, i32); 4] {
    let mut coords = [(0, 0); 4];
    let mut i = 0;
    while i < 4 {
        coords[i] = (20, 20 + (i as i32) * 40);
        i += 1;
    }
    coords
}

const DISARMED_COORD: [(i32, i32); 1] = [(20, 80)];
const AXES_COORD: [(i32, i32); 4] = axes_coords();

/// # Page
/// The currently displayed page (with the text coordinates). For now it can be:
/// * "DISARMED"
/// * The 4 axes
enum Page {
    Disarmed([(i32, i32); 1]),
    Axes([(i32, i32); 4]),
}

impl Page {
    /// # Coordinates
    /// Returns the text coordinates of the currently displayed text, in an array.
    fn coords(&self) -> &[(i32, i32)] {
        match self {
            Page::Disarmed(coords) => coords,
            Page::Axes(coords) => coords,
        }
    }
}

/// # Screen
/// Used to display text on the screen.
pub struct Screen {
    /// LCD screen handle
    pub display: Lcd,
    /// Currently displayed page
    page: Page,
    /// Style to use for the text.
    style: MonoTextStyle<'static, Rgb565>,
}

impl Screen {
    /// Create a `Screen` instance.
    pub fn new(display: Lcd) -> Self {
        let style = MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(Rgb565::WHITE)
            .background_color(Rgb565::BLACK)
            .build();

        let page = Page::Axes(AXES_COORD);

        Screen {
            display,
            page,
            style,
        }
    }

    /// # Draw Axes
    /// * Define the display style (here just text).
    /// * Store the axes with their name to be displayed.
    /// * Write the data and send it to the LCD screen.
    pub fn draw_axes(&mut self, throttle: u16, yaw: u16, pitch: u16, roll: u16) {
        self.erase(); // Overwrite "DISARMED" with blanks, then show the 4 axes

        // Write the axes into the screen
        let axes: [(&str, u16); 4] = [
            ("Throttle", throttle),
            ("Yaw", yaw),
            ("Pitch", pitch),
            ("Roll", roll),
        ];

        for (i, (label, value)) in axes.iter().enumerate() {
            let mut buf: String<32> = String::new();
            let _ = write!(buf, "{:<10}{:>4}", label, value);
            let y = 20 + (i as i32) * 40;
            let _ = Text::with_baseline(&buf, Point::new(20, y), self.style, Baseline::Top)
                .draw(&mut self.display);
        }

        self.page = Page::Axes(AXES_COORD); // New current page is the 4 axes
    }

    /// # Draw "DISARMED"
    /// Overwrite the previous display with blanks, then show DISARMED
    pub fn draw_disarmed(&mut self) {
        self.erase(); // Overwrite the 4 axes lines with blanks, then show DISARMED

        // Write "DISARMED" on the screen
        let _ = Text::with_baseline("DISARMED", Point::new(20, 80), self.style, Baseline::Top)
            .draw(&mut self.display);

        self.page = Page::Disarmed(DISARMED_COORD); // New current page is "DISARMED"
    }

    /// # Erase Current Page
    /// Get the display coordinates of the current page and for each coordinates:
    /// * Create an empty String buffer
    /// * Write the empty buffer into the screen
    ///
    /// This overwrites the previous text at these coordinates.
    fn erase(&mut self) {
        for &(x, y) in self.page.coords() {
            let mut buf: String<32> = String::new();
            let _ = write!(buf, "{:<14}", "");
            let _ = Text::with_baseline(&buf, Point::new(x, y), self.style, Baseline::Top)
                .draw(&mut self.display);
        }
    }
}
