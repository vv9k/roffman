use crate::_macro::{ENDL, SECTION_HEADER, SPACE, SUB_HEADER};
use crate::{
    node::RoffNodeInner, write_quoted_if_whitespace, IntoRoffNode, RoffError, RoffText, Roffable,
};

use std::io::Write;

#[derive(Clone, Debug)]
/// A single section of the ROFF document.
pub struct Section {
    title: RoffText,
    subtitle: Option<RoffText>,
    nodes: Vec<RoffNodeInner>,
}

impl Section {
    /// Create a new section with `title` and `content`.
    pub fn new<I, R>(title: impl Roffable, content: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoRoffNode,
    {
        Self {
            title: title.roff(),
            subtitle: None,
            nodes: content
                .into_iter()
                .map(|r| r.into_roff().into_inner())
                .collect(),
        }
    }

    /// Set the sub heading of this section.
    pub fn subtitle(mut self, subtitle: impl Roffable) -> Self {
        self.subtitle = Some(subtitle.roff());
        self
    }

    pub(crate) fn render<W: Write>(
        &self,
        writer: &mut W,
        was_text: bool,
    ) -> Result<bool, RoffError> {
        if was_text {
            writer.write_all(ENDL)?;
        }
        writer.write_all(SECTION_HEADER)?;
        writer.write_all(SPACE)?;
        write_quoted_if_whitespace(&self.title, writer)?;
        writer.write_all(ENDL)?;
        if let Some(subtitle) = &self.subtitle {
            writer.write_all(SUB_HEADER)?;
            writer.write_all(SPACE)?;
            write_quoted_if_whitespace(subtitle, writer)?;
            writer.write_all(ENDL)?;
        }

        let mut was_text = false;
        for node in &self.nodes {
            was_text = node.render(writer, was_text)?;
        }

        Ok(was_text)
    }
}
