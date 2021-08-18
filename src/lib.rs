use std::io::{self, Write};
use thiserror::Error;

const COMMA: &[u8] = b".\n";
const SPACE: &[u8] = b" ";
const QUOTE: &[u8] = b"\"";
const ENDL: &[u8] = b"\n";
const BOLD: &[u8] = b"\\fB";
const ITALIC: &[u8] = b"\\fI";
const FONT_END: &[u8] = b"\\fR";
const SECTION_HEADER: &[u8] = b".SH";
const TITLE_HEADER: &[u8] = b".TH";
const PARAGRAPH: &[u8] = b".P";
const INDENTED_PARAGRAPH: &[u8] = b".IP";
const TAGGED_PARAGRAPH: &[u8] = b".TP";

#[derive(Error, Debug)]
pub enum RoffError {
    #[error("Failed to render roff as string - `{0}`")]
    StringRenderFailed(String),
    #[error("Failed to render roff - `{0}`")]
    RenderFailed(#[from] io::Error),
}

fn escape<T: AsRef<str>>(text: T) -> String {
    let text = text.as_ref();
    let text = text.replace('.', "\\.");
    text.replace('-', "\\-")
}

fn write_quoted<W: Write>(content: &[u8], writer: &mut W) -> io::Result<()> {
    writer.write(QUOTE)?;
    writer.write(content)?;
    writer.write(QUOTE).map(|_| ())
}

pub struct Roff {
    title: String,
    date: Option<String>,
    section: u8,
    sections: Vec<Section>,
}

impl Roff {
    pub fn new<T: AsRef<str>>(title: T, section: u8) -> Self {
        Self {
            title: escape(title),
            date: None,
            section,
            sections: vec![],
        }
    }

    pub fn to_string(&self) -> Result<String, RoffError> {
        let mut writer = std::io::BufWriter::new(vec![]);
        self.render(&mut writer)
            .map_err(|e| RoffError::StringRenderFailed(e.to_string()))?;
        String::from_utf8(
            writer
                .into_inner()
                .map_err(|e| RoffError::StringRenderFailed(e.to_string()))?,
        )
        .map_err(|e| RoffError::StringRenderFailed(e.to_string()))
    }

    pub fn date<D: Into<String>>(mut self, date: D) -> Self {
        self.date = Some(date.into());
        self
    }

    pub fn section<T, C>(mut self, title: T, content: C) -> Self
    where
        T: AsRef<str>,
        C: IntoIterator<Item = RoffNode>,
    {
        self.sections.push(Section {
            title: escape(title),
            nodes: content.into_iter().collect(),
        });
        self
    }

    pub fn render<W: Write>(&self, writer: &mut W) -> Result<(), RoffError> {
        writer.write(TITLE_HEADER)?;
        writer.write(SPACE)?;
        write_quoted(self.title.as_bytes(), writer)?;
        writer.write(format!(" \"{}\" ", self.section).as_bytes())?;
        if let Some(date) = &self.date {
            write_quoted(date.as_bytes(), writer)?;
        }
        writer.write(ENDL)?;
        writer.write(COMMA)?;

        for section in &self.sections {
            section.render(writer)?;
        }

        Ok(())
    }
}

pub struct Section {
    title: String,
    nodes: Vec<RoffNode>,
}

impl Section {
    pub fn render<W: Write>(&self, writer: &mut W) -> Result<(), RoffError> {
        writer.write(SECTION_HEADER)?;
        writer.write(SPACE)?;
        write_quoted(self.title.as_bytes(), writer)?;
        writer.write(ENDL)?;
        writer.write(COMMA)?;

        for node in &self.nodes {
            node.render(writer)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum Style {
    Bold,
    Italic,
    Normal,
}

impl Default for Style {
    fn default() -> Self {
        Style::Normal
    }
}

pub struct RoffText {
    content: String,
    style: Style,
}

impl RoffText {
    pub fn new<C: AsRef<str>>(content: C, style: Option<Style>) -> Self {
        Self {
            content: escape(content),
            style: style.unwrap_or_default(),
        }
    }

    pub fn bold(mut self) -> Self {
        self.style = Style::Bold;
        self
    }

    pub fn italic(mut self) -> Self {
        self.style = Style::Italic;
        self
    }

    pub fn render<W: Write>(&self, writer: &mut W) -> Result<(), RoffError> {
        let styled = match self.style {
            Style::Bold => {
                writer.write(BOLD)?;
                true
            }
            Style::Italic => {
                writer.write(ITALIC)?;
                true
            }
            Style::Normal => false,
        };

        writer.write(self.content.as_bytes())?;
        if styled {
            writer.write(FONT_END)?;
        }

        Ok(())
    }
}

pub enum RoffNode {
    Paragraph(Vec<RoffText>),
    IndentedParagraph {
        content: Vec<RoffText>,
        indentation: Option<u8>,
    },
    TaggedParagraph {
        content: Vec<RoffText>,
        tag: RoffText,
    },
}

impl RoffNode {
    pub fn paragraph<C>(content: C) -> Self
    where
        C: IntoIterator<Item = RoffText>,
    {
        Self::Paragraph(content.into_iter().collect())
    }

    pub fn indented_paragraph<C>(content: C, indentation: Option<u8>) -> Self
    where
        C: IntoIterator<Item = RoffText>,
    {
        Self::IndentedParagraph {
            content: content.into_iter().collect(),
            indentation,
        }
    }

    pub fn tagged_paragraph<C>(content: C, tag: RoffText) -> Self
    where
        C: IntoIterator<Item = RoffText>,
    {
        Self::TaggedParagraph {
            content: content.into_iter().collect(),
            tag,
        }
    }

    pub fn render<W: Write>(&self, writer: &mut W) -> Result<(), RoffError> {
        match self {
            RoffNode::Paragraph(content) => {
                writer.write(PARAGRAPH)?;
                writer.write(ENDL)?;
                for text in content {
                    text.render(writer)?;
                }
                writer.write(ENDL)?;
                writer.write(COMMA)?;
            }
            RoffNode::IndentedParagraph {
                content,
                indentation,
            } => {
                writer.write(INDENTED_PARAGRAPH)?;
                if let Some(indentation) = indentation {
                    writer.write(format!(" \"\" {}", indentation).as_bytes())?;
                }
                writer.write(ENDL)?;
                for text in content {
                    text.render(writer)?;
                }
                writer.write(ENDL)?;
                writer.write(COMMA)?;
            }
            RoffNode::TaggedParagraph { content, tag } => {
                writer.write(TAGGED_PARAGRAPH)?;
                writer.write(ENDL)?;
                tag.render(writer)?;
                writer.write(ENDL)?;

                for text in content {
                    text.render(writer)?;
                }

                writer.write(ENDL)?;
                writer.write(COMMA)?;
            }
        }

        Ok(())
    }
}

pub trait Roffable {
    fn roff(&self) -> RoffText;
}

impl Roffable for &str {
    fn roff(&self) -> RoffText {
        RoffText::new(self.to_string(), None)
    }
}

impl Roffable for String {
    fn roff(&self) -> RoffText {
        RoffText::new(self.clone(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_roffs() {
        let roff = Roff::new("test", 1)
            .section(
                "test section 1",
                vec![RoffNode::paragraph(vec![
                    "this is some very ".roff(),
                    "special".roff().bold(),
                    " text".roff(),
                ])],
            )
            .section(
                "test section 2",
                vec![RoffNode::indented_paragraph(
                    vec![
                        "Lorem ipsum".roff().italic(),
                        " dolor sit amet, consectetur adipiscing elit. Vivamus quis malesuada eros."
                            .roff(),
                    ],
                    Some(4),
                )],
            )
            .section(
                "test section 3",
                vec![RoffNode::tagged_paragraph(
                    vec!["tagged paragraph with some content".roff()],
                    "paragraph title".roff().bold(),
                )],
            );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH "test" "1" 
.
.SH "test section 1"
.
.P
this is some very \fBspecial\fR text
.
.SH "test section 2"
.
.IP "" 4
\fILorem ipsum\fR dolor sit amet, consectetur adipiscing elit\. Vivamus quis malesuada eros\.
.
.SH "test section 3"
.
.TP
\fBparagraph title\fR
tagged paragraph with some content
.
"#,
            rendered
        )
    }
}
