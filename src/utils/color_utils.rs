use colored::Colorize;

use crate::utils::color_config;

#[allow(unused)]
pub trait Colors {
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn red(&self) -> String;
    fn orange(&self) -> String;
    fn bright_orange(&self) -> String;
    fn blue(&self) -> String;
    fn dark_blue(&self) -> String;
    fn purple(&self) -> String;
    fn black(&self) -> String;
    fn white(&self) -> String;
    fn gray(&self) -> String;
}

impl Colors for String {
    fn green(&self) -> Self {
        apply_color(&self, color_config::GREEN)
    }

    fn yellow(&self) -> Self {
        apply_color(&self, color_config::YELLOW)
    }

    fn red(&self) -> Self {
        apply_color(&self, color_config::RED)
    }

    fn orange(&self) -> Self {
        apply_color(&self, color_config::ORANGE)
    }

    fn bright_orange(&self) -> Self {
        apply_color(&self, color_config::BRIGHT_ORANGE)
    }

    fn blue(&self) -> Self {
        apply_color(&self, color_config::BLUE)
    }

    fn dark_blue(&self) -> Self {
        apply_color(&self, color_config::DARK_BLUE)
    }

    fn purple(&self) -> Self {
        apply_color(&self, color_config::PURPLE)
    }

    fn black(&self) -> Self {
        apply_color(&self, color_config::BLACK)
    }

    fn white(&self) -> Self {
        apply_color(&self, color_config::WHITE)
    }

    fn gray(&self) -> Self {
        apply_color(&self, color_config::GRAY)
    }
}

impl Colors for &str {
    fn green(&self) -> String {
        apply_color(&self, color_config::GREEN)
    }

    fn yellow(&self) -> String {
        apply_color(&self, color_config::YELLOW)
    }

    fn red(&self) -> String {
        apply_color(&self, color_config::RED)
    }

    fn orange(&self) -> String {
        apply_color(&self, color_config::ORANGE)
    }

    fn bright_orange(&self) -> String {
        apply_color(&self, color_config::BRIGHT_ORANGE)
    }

    fn blue(&self) -> String {
        apply_color(&self, color_config::BLUE)
    }

    fn dark_blue(&self) -> String {
        apply_color(&self, color_config::DARK_BLUE)
    }

    fn purple(&self) -> String {
        apply_color(&self, color_config::PURPLE)
    }

    fn black(&self) -> String {
        apply_color(&self, color_config::BLACK)
    }

    fn white(&self) -> String {
        apply_color(&self, color_config::WHITE)
    }

    fn gray(&self) -> String {
        apply_color(&self, color_config::GRAY)
    }
}

#[allow(unused)]
pub trait CustomColor {
    fn custom_color(&self, color: (u8, u8, u8)) -> String;
}

impl CustomColor for String {
    fn custom_color(&self, color: (u8, u8, u8)) -> String {
        apply_color(&self, color)
    }
}

impl CustomColor for &str {
    fn custom_color(&self, color: (u8, u8, u8)) -> String {
        apply_color(&self, color)
    }
}

#[allow(unused)]
pub fn lerp_color(a: (u8, u8, u8), b: (u8, u8, u8), value: f32) -> (u8, u8, u8) {
    let result_r = a.0 as i16 + ((b.0 as i16 - a.0 as i16) as f32 * value) as i16;
    let result_g = a.1 as i16 + ((b.1 as i16 - a.1 as i16) as f32 * value) as i16;
    let result_b = a.2 as i16 + ((b.2 as i16 - a.2 as i16) as f32 * value) as i16;
    (result_r as u8, result_g as u8, result_b as u8)
}

fn apply_color(content: &str, color: (u8, u8, u8)) -> String {
    content.truecolor(color.0, color.1, color.2).to_string()
}
