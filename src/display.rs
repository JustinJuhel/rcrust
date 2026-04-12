use core::fmt::Write;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use heapless::String;

/// # Draw Axes
/// * Define the display style (here just text).
/// * Store the axes with their name to be displayed.
/// * Write the data and send it to the LCD screen.
pub fn draw_axes(
    display: &mut impl DrawTarget<Color = Rgb565>,
    throttle: u16,
    yaw: u16,
    pitch: u16,
    roll: u16,
) {
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

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
        let _ = Text::with_baseline(&buf, Point::new(20, y), style, Baseline::Top).draw(display);
    }
}
