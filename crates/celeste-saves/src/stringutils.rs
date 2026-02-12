use once_cell::sync::Lazy;

#[derive(Debug, PartialEq)]
pub enum ColorMode {
    NoColor,
    TermColor,
    HtmlColor,
}
pub use ColorMode::*;
pub static COLOR_MODE: Lazy<ColorMode> = Lazy::new(|| {
    match (
        std::env::var("CELESTE_SAVE_COLOR"),
        std::env::var("NO_COLOR"),
        atty::is(atty::Stream::Stdout),
    ) {
        (Ok(s), _, _) if s == "ON" => TermColor,
        (Ok(s), _, _) if s == "HTML" => HtmlColor,
        (_, Err(_), true) => TermColor,
        (_, _, _) => NoColor,
    }
});

pub trait StringUtils {
    fn with_ansi(&self, prefix: u8, suffix: u8) -> String;
    fn color(&self, color: AnsiColor) -> String;
    fn background(&self, color: AnsiColor) -> String;
    fn underline(&self) -> String;
    fn invert(&self) -> String;
    fn pad_start(&self, len: usize) -> String;
    fn pad_end(&self, len: usize) -> String;
}

// see https://misc.flogisoft.com/bash/tip_colors_and_formatting

impl StringUtils for &str {
    fn with_ansi(&self, prefix: u8, suffix: u8) -> String {
        format!("\x1B[{}m{}\x1B[{}m", prefix, self, suffix)
    }

    fn color(&self, color: AnsiColor) -> String {
        match *COLOR_MODE {
            TermColor => self.with_ansi(30 + color.offset(), 39),
            HtmlColor => format!(
                "<span style=\"color: {};\">{}</span>",
                color.css_color(),
                self
            ),
            NoColor => self.to_string(),
        }
    }

    fn background(&self, color: AnsiColor) -> String {
        match *COLOR_MODE {
            TermColor => self.with_ansi(40 + color.offset(), 49),
            HtmlColor => format!(
                "<span style=\"background-color: {};\">{}</span>",
                color.css_color(),
                self
            ),
            NoColor => self.to_string(),
        }
    }

    fn underline(&self) -> String {
        match *COLOR_MODE {
            TermColor => self.with_ansi(4, 24),
            HtmlColor => format!(
                "<span style=\"text-decoration: underline;\">{}</span>",
                self
            ),
            NoColor => self.to_string(),
        }
    }

    fn invert(&self) -> String {
        match *COLOR_MODE {
            TermColor => self.with_ansi(4, 24),
            HtmlColor => format!("<span style=\"font-weight: bold;\">{}</span>", self),
            NoColor => self.to_string(),
        }
    }

    fn pad_start(&self, len: usize) -> String {
        format!("{:>0len$}", self, len = len)
    }

    fn pad_end(&self, len: usize) -> String {
        format!("{:<0len$}", self, len = len)
    }
}
impl StringUtils for String {
    fn with_ansi(&self, prefix: u8, suffix: u8) -> String {
        self.as_str().with_ansi(prefix, suffix)
    }

    fn color(&self, color: AnsiColor) -> String {
        self.as_str().color(color)
    }

    fn background(&self, color: AnsiColor) -> String {
        self.as_str().background(color)
    }

    fn underline(&self) -> String {
        self.as_str().underline()
    }
    fn invert(&self) -> String {
        self.as_str().invert()
    }
    fn pad_start(&self, len: usize) -> String {
        self.as_str().pad_start(len)
    }

    fn pad_end(&self, len: usize) -> String {
        self.as_str().pad_end(len)
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum AnsiColor {
    Default,
    Black,
    White,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    LightGray,
    DarkGray,
    Cyan,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
}

pub use self::AnsiColor::*;

impl AnsiColor {
    fn offset(self) -> u8 {
        match self {
            Default => 9,
            Black => 0,
            White => 67,
            Red => 61,
            Green => 62,
            Yellow => 63,
            Blue => 64,
            Magenta => 65,
            LightGray => 7,
            DarkGray => 60,
            Cyan => 66,
            DarkRed => 1,
            DarkGreen => 2,
            DarkYellow => 3,
            DarkBlue => 4,
            DarkMagenta => 5,
            DarkCyan => 6,
        }
    }

    fn css_color(self) -> &'static str {
        match self {
            Default => "default",
            Black => "#000",
            White => "#FFF",
            Red => "#EF2929",
            Green => "#8AE234",
            Yellow => "#FCE94F",
            Blue => "#32AFFF",
            Magenta => "#AD7FA8",
            LightGray => "#D3D7CF",
            DarkGray => "#555753",
            Cyan => "#34E2E2",
            DarkRed => "#C00",
            DarkGreen => "#4E9A06",
            DarkYellow => "#C4A000",
            DarkBlue => "#729FCF",
            DarkMagenta => "#75507B",
            DarkCyan => "#06989A",
        }
    }
}
