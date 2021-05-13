//! LCD UI

use embedded_graphics::mono_font::ascii::FONT_6X12;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::primitives::{
    Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle,
};
use embedded_graphics::text::Text;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use core::fmt::Write;
use heapless::String;

use crate::consts::{UI_HEIGHT, UI_WIDTH};
use crate::{delay::AsmDelay, lcd::*, model::*};

pub struct LcdUI {
    lcd: Lcd<AsmDelay, 0>,
}

const TOP_TEXT_HEIGHT: u32 = 20;
const GRAPH_HEIGHT: u32 = UI_HEIGHT as u32 - TOP_TEXT_HEIGHT;

impl LcdUI {
    pub fn new(lcd: Lcd<AsmDelay, 0>) -> Self {
        LcdUI { lcd }
    }

    pub fn init(&mut self) -> Result<(), LcdError> {
        self.lcd.init()?;
        self.lcd.set_rotation(Rotation::R90)?;
        self.lcd.clear(Rgb565::BLACK)
    }

    pub fn render(&mut self, model: &UIModel) -> Result<(), LcdError> {
        let style = MonoTextStyleBuilder::new()
            .font(&FONT_6X12)
            .text_color(Rgb565::YELLOW)
            .background_color(Rgb565::BLACK)
            .build();

        let mut sbuf: String<64> = String::new();
        write!(
            sbuf,
            "HR {:>3.1} ",
            model
                .ir
                .heart_rate_bpm
                .or(model.r.heart_rate_bpm)
                .unwrap_or(0.0)
        )?;
        Text::new(&sbuf, Point::new(10, 10), style).draw(&mut self.lcd)?;

        sbuf.clear();
        write!(sbuf, "SPO2 {:>2.1} ", model.spo2())?;
        Text::new(&sbuf, Point::new(100, 10), style).draw(&mut self.lcd)?;

        self.lcd
            .fill_solid(
                &Rectangle::new(
                    Point::new(0, TOP_TEXT_HEIGHT as i32),
                    Size::new(UI_WIDTH as u32, GRAPH_HEIGHT),
                ),
                Rgb565::BLACK,
            )
            .unwrap();

        self.render_ac_sample_data(&model.r, Rgb565::RED)?;
        self.render_ac_sample_data(&model.ir, Rgb565::BLUE)?;

        Ok(())
    }

    pub fn render_ac_sample_data(
        &mut self,
        samples: &Max3012SampleData,
        color: Rgb565,
    ) -> Result<(), LcdError> {
        // leave some space at the top for text output
        let scale = (GRAPH_HEIGHT - 1) as f32 / (samples.ac_max - samples.ac_min);
        let line_style = PrimitiveStyle::with_stroke(color, 1);

        #[inline]
        fn sx(i: usize) -> i32 {
            i as i32 * 2
        }

        let sy = |s: f32| (UI_HEIGHT - 1) as i32 - (s * scale) as i32;

        let mut p0_opt: Option<Point> = None;
        for (i, acs) in samples.ac.iter().enumerate() {
            let x = sx(i);
            let y = sy(acs - samples.ac_min);
            let p = Point::new(x, y);

            for p0 in p0_opt {
                Line::new(p0, p)
                    .into_styled(line_style)
                    .draw(&mut self.lcd)?;
            }

            p0_opt = Some(p);
        }

        let peak_style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .fill_color(Rgb565::YELLOW)
            .stroke_width(1)
            .build();

        let hb_cir = |i, v| {
            let x = sx(i);
            let y = sy(v - samples.ac_min);
            Circle::new(Point::new(x as i32, y as i32), 5).into_styled(peak_style)
        };

        for hb in &samples.heartbeats {
            hb_cir(hb.high_idx, hb.high_value).draw(&mut self.lcd)?;
            hb_cir(hb.low_idx, hb.low_value).draw(&mut self.lcd)?;
        }

        Ok(())
    }
}
