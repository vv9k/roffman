use crate::_macro::{BOLD, FONT_END, ITALIC};
use crate::{escape, RoffError, Roffable};

use std::io::Write;

#[derive(Copy, Clone, Debug, PartialEq)]
/// Style that can be applied to [`RoffText`](RoffText).
pub enum FontStyle {
    Bold,
    Italic,
    Roman,
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle::Roman
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Wrapper type for styled text in ROFF. The most basic unit of text used in the document. It can
/// be styled with various [`FontStyle`s](FontStyle) and will escape it's contents on creation so
/// that they are safe to render and will be correctly displayed on various viewers.
pub struct RoffText {
    content: String,
    style: FontStyle,
}

impl RoffText {
    /// Create a new `RoffText` with `content` and optional font `style`. The text will automatically
    /// be escaped on initialization.
    pub fn new<C: AsRef<str>>(content: C, style: Option<FontStyle>) -> Self {
        Self {
            content: escape(content),
            style: style.unwrap_or_default(),
        }
    }

    /// Set the style of this text to bold.
    pub fn bold(mut self) -> Self {
        self.style = FontStyle::Bold;
        self
    }

    /// Set the style of this text to italic.
    pub fn italic(mut self) -> Self {
        self.style = FontStyle::Italic;
        self
    }

    /// Return the underlying escaped text.
    pub(crate) fn content(&self) -> &str {
        &self.content
    }

    pub(crate) fn render<W: Write>(&self, writer: &mut W) -> Result<(), RoffError> {
        let styled = match self.style {
            FontStyle::Bold => {
                writer.write_all(BOLD)?;
                true
            }
            FontStyle::Italic => {
                writer.write_all(ITALIC)?;
                true
            }
            FontStyle::Roman => false,
        };

        writer.write_all(self.content.as_bytes())?;
        if styled {
            writer.write_all(FONT_END)?;
        }

        Ok(())
    }
}

impl Roffable for RoffText {
    fn roff(&self) -> RoffText {
        self.clone()
    }
}
