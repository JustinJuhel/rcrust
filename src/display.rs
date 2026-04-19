use core::fmt::Write;

use embedded_graphics::mono_font::jis_x0201::FONT_10X20;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use heapless::String;

use crate::init::Lcd;

const ARM_DISARM_COORD: [(i32, i32); 1] = [(20, 80)];

/// # Page
/// The currently displayed page (with the text coordinates). For now it can be:
/// * "DISARMED"
/// * The 4 axes
enum Page {
    ArmDisarm([(i32, i32); 1]),
}

impl Page {
    /// # Coordinates
    /// Returns the text coordinates of the currently displayed text, in an array.
    fn coords(&self) -> &[(i32, i32)] {
        match self {
            Page::ArmDisarm(coords) => coords,
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

        let page = Page::ArmDisarm(ARM_DISARM_COORD);

        Screen {
            display,
            page,
            style,
        }
    }

    /// # Draw "DISARMED"
    /// Overwrite the previous display with blanks, then show DISARMED
    pub fn draw_disarmed(&mut self, armed: bool) {
        // Write "DISARMED" or "ARMED" on the screen
        let text = if armed { "ARMED" } else { "DISARMED" };
        let _ = Text::with_baseline(text, Point::new(20, 80), self.style, Baseline::Top)
            .draw(&mut self.display);

        self.page = Page::ArmDisarm(ARM_DISARM_COORD); // New current page is "DISARMED"
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
