//! # roffman - create ROFF man pages in rust with ease!
//!
//!
//! ## Example usage
//! ```
//! use roffman::{IntoRoffNode, Roff, Roffable, RoffNode};
//!
//! fn main() {
//!     let roff = Roff::new("roffman", 7).section(
//!         "BASIC USAGE",
//!         vec![
//!             RoffNode::paragraph(vec![
//!                 "This is how you create a basic paragraph using roffman.",
//!             ]),
//!             RoffNode::indented_paragraph(
//!                 vec![
//!                     "This line should be slightly indented to the ".roff(),
//!                     "right.".roff().bold(),
//!                 ],
//!                 Some(4),
//!             ),
//!             RoffNode::text("And some example "),
//!             RoffNode::text("code".roff().italic()),
//!             RoffNode::text(":"),
//!             RoffNode::example(vec![
//!                 r#"
//! impl Roffable for u8 {
//!     fn roff(&self) -> RoffText {
//!         self.to_string().roff()
//!     }
//! }"#,
//!             ]),
//!         ],
//!     );
//!
//!     let rendered = roff.to_string().unwrap();
//!     println!("{}", rendered);
//! }
//! ```
//!
//! will produce:
//! ```roff
//! .TH "roffman" "7"
//! .
//! .SH "BASIC USAGE"
//!
//! .P
//! This is how you create a basic paragraph using roffman\.
//! .
//! .IP "" 4
//! This line should be slightly indented to the \fBright\.\fR
//! .And some example \fIcode\fR:
//! .EX
//!
//! impl Roffable for u8 {
//!     fn roff(&self) \-> RoffText {
//!         self\.to_string()\.roff()
//!     }
//! }
//! .EE
//! .
//! ```

use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::io::{self, Write};

const COMMA: &[u8] = b"\n.";
const SPACE: &[u8] = b" ";
const QUOTE: &[u8] = b"\"";
const ENDL: &[u8] = b"\n";
const BOLD: &[u8] = b"\\fB";
const ITALIC: &[u8] = b"\\fI";
const FONT_END: &[u8] = b"\\fR";
const SECTION_HEADER: &[u8] = b".SH";
const SUB_HEADER: &[u8] = b".SS";
const TITLE_HEADER: &[u8] = b".TH";
const PARAGRAPH: &[u8] = b".P";
const INDENTED_PARAGRAPH: &[u8] = b".IP";
const TAGGED_PARAGRAPH: &[u8] = b".TP";
const NESTED_START: &[u8] = b".RS";
const NESTED_END: &[u8] = b".RE";
const EXAMPLE_START: &[u8] = b".EX";
const EXAMPLE_END: &[u8] = b".EE";

#[derive(Debug)]
/// An error type returned by the functions used in this crate.
pub enum RoffError {
    StringRenderFailed(String),
    RenderFailed(io::Error),
}

impl fmt::Display for RoffError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RoffError::StringRenderFailed(err) => {
                write!(f, "Failed to render ROFF to string - `{}`", err)
            }
            RoffError::RenderFailed(err) => write!(f, "Failed to render ROFF - `{}`", err),
        }
    }
}

impl Error for RoffError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RoffError::RenderFailed(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for RoffError {
    fn from(err: io::Error) -> Self {
        Self::RenderFailed(err)
    }
}

fn escape<T: AsRef<str>>(text: T) -> String {
    let text = text.as_ref();
    let text = text.replace('.', "\\.");
    text.replace('-', "\\-")
}

fn write_quoted(roff: &RoffText, writer: &mut impl Write) -> Result<(), RoffError> {
    writer.write(QUOTE)?;
    roff.render(writer)?;
    writer.write(QUOTE)?;
    Ok(())
}

#[derive(Clone, Debug)]
/// Represents a ROFF document that can be rendered and displayed
/// with tools like [`man`](https://man7.org/linux/man-pages/man1/man.1.html).
pub struct Roff {
    title: RoffText,
    date: Option<RoffText>,
    section: u8,
    sections: Vec<Section>,
}

impl Roff {
    /// Create a new `Roff` with a `title` and a `section`.
    pub fn new(title: impl Roffable, section: u8) -> Self {
        Self {
            title: title.roff(),
            date: None,
            section,
            sections: vec![],
        }
    }

    /// Renders this roff to a `String` returning an error if a write fails or the rendered
    /// output contains invalid UTF-8 byte sequences.
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

    /// Builder method for adding a date to this roff.
    pub fn date(mut self, date: impl Roffable) -> Self {
        self.date = Some(date.roff());
        self
    }

    /// Add an already defined section to this roff.
    pub fn add_section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    /// Builder method for adding a new section to this roff.
    pub fn section<I, R>(self, title: impl Roffable, content: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoRoffNode,
    {
        self.add_section(Section::new(title, content))
    }

    fn write_title(&self, writer: &mut impl Write) -> Result<(), RoffError> {
        writer.write(SPACE)?;
        write_quoted(&self.title, writer)
    }

    fn write_section(&self, writer: &mut impl Write) -> Result<(), RoffError> {
        writer.write(SPACE)?;
        write_quoted(&self.section.roff(), writer)
    }

    fn write_date(&self, writer: &mut impl Write) -> Result<(), RoffError> {
        if let Some(date) = &self.date {
            writer.write(SPACE)?;
            write_quoted(&date, writer)?;
        }
        Ok(())
    }

    fn write_title_header(&self, writer: &mut impl Write) -> Result<(), RoffError> {
        writer.write(TITLE_HEADER)?;
        self.write_title(writer)?;
        self.write_section(writer)?;
        self.write_date(writer)?;
        writer.write(COMMA)?;
        Ok(())
    }

    /// Renders this `Roff` to a `writer` returning an error if any of the writes fails.
    pub fn render<W: Write>(&self, writer: &mut W) -> Result<(), RoffError> {
        self.write_title_header(writer)?;

        for section in &self.sections {
            section.render(writer)?;
        }

        Ok(())
    }
}

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

    fn render<W: Write>(&self, writer: &mut W) -> Result<(), RoffError> {
        writer.write(ENDL)?;
        writer.write(SECTION_HEADER)?;
        writer.write(SPACE)?;
        write_quoted(&self.title, writer)?;
        writer.write(ENDL)?;
        if let Some(subtitle) = &self.subtitle {
            writer.write(SUB_HEADER)?;
            writer.write(SPACE)?;
            writer.write(QUOTE)?;
            subtitle.render(writer)?;
            writer.write(QUOTE)?;
            writer.write(ENDL)?;
        }

        for node in &self.nodes {
            node.render(writer, false)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
/// Style that can be applied to [`RoffText`](RoffText)
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

#[derive(Clone, Debug, Default)]
/// Wrapper type for styled text in ROFF.
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

    /// Set the style of this text to bold.
    pub fn bold(mut self) -> Self {
        self.style = Style::Bold;
        self
    }

    /// Set the style of this text to italic.
    pub fn italic(mut self) -> Self {
        self.style = Style::Italic;
        self
    }

    fn render<W: Write>(&self, writer: &mut W) -> Result<(), RoffError> {
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

#[derive(Clone, Debug)]
/// Base struct used to create ROFFs.
pub struct RoffNode(RoffNodeInner);

impl RoffNode {
    #[inline]
    fn into_inner(self) -> RoffNodeInner {
        self.0
    }

    pub fn text(content: impl Roffable) -> Self {
        Self(RoffNodeInner::Text(content.roff()))
    }

    /// Creates a new paragraph node.
    pub fn paragraph<I, R>(content: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoRoffNode,
    {
        Self(RoffNodeInner::Paragraph(
            content
                .into_iter()
                .map(|item| item.into_roff().into_inner())
                .collect(),
        ))
    }

    /// Creates a new indented paragraph node.
    pub fn indented_paragraph<I, R>(content: I, indentation: Option<u8>) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoRoffNode,
    {
        Self(RoffNodeInner::IndentedParagraph {
            content: content
                .into_iter()
                .map(|item| item.into_roff().into_inner())
                .collect(),
            indentation,
        })
    }

    /// Creates a new paragraph node with a title.
    pub fn tagged_paragraph<I, R, T>(content: I, title: T) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoRoffNode,
        T: Roffable,
    {
        Self(RoffNodeInner::TaggedParagraph {
            content: content
                .into_iter()
                .map(|item| item.into_roff().into_inner())
                .collect(),
            title: title.roff(),
        })
    }

    /// Creates a new example node.
    pub fn example<I, R>(content: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Roffable,
    {
        Self(RoffNodeInner::Example(
            content.into_iter().map(|item| item.roff()).collect(),
        ))
    }
}

#[derive(Clone, Debug)]
/// Base struct used to create ROFFs.
enum RoffNodeInner {
    /// The most basic node type, contains only text with style.
    Text(RoffText),
    /// A simple paragraph that can contain nested items.
    Paragraph(Vec<RoffNodeInner>),
    /// Indented paragraph that can contain nested items. If no indentation is provided the default
    /// is `4`.
    IndentedParagraph {
        content: Vec<RoffNodeInner>,
        indentation: Option<u8>,
    },
    /// Paragraph with a title.
    TaggedParagraph {
        content: Vec<RoffNodeInner>,
        title: RoffText,
    },
    /// An example block where text is monospaced.
    Example(Vec<RoffText>),
}

impl RoffNodeInner {
    /// Returns `true` if the node is the [`RoffNode::Text`](RoffNode::Text) variant.
    pub fn is_text(&self) -> bool {
        if let &RoffNodeInner::Text(_) = self {
            true
        } else {
            false
        }
    }

    fn render<W: Write>(&self, writer: &mut W, nested: bool) -> Result<(), RoffError> {
        if nested {
            writer.write(ENDL)?;
            writer.write(NESTED_START)?;
        }
        match self {
            RoffNodeInner::Text(text) => {
                let styled = match text.style {
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

                writer.write(text.content.as_bytes())?;
                if styled {
                    writer.write(FONT_END)?;
                }
            }
            RoffNodeInner::Paragraph(content) => {
                writer.write(ENDL)?;
                writer.write(PARAGRAPH)?;
                writer.write(ENDL)?;
                for node in content {
                    node.render(writer, !node.is_text())?;
                }
                writer.write(COMMA)?;
            }
            RoffNodeInner::IndentedParagraph {
                content,
                indentation,
            } => {
                writer.write(ENDL)?;
                writer.write(INDENTED_PARAGRAPH)?;
                if let Some(indentation) = indentation {
                    writer.write(format!(" \"\" {}", indentation).as_bytes())?;
                }
                writer.write(ENDL)?;
                for node in content {
                    node.render(writer, !node.is_text())?;
                }
                writer.write(COMMA)?;
            }
            RoffNodeInner::TaggedParagraph {
                content,
                title: tag,
            } => {
                writer.write(ENDL)?;
                writer.write(TAGGED_PARAGRAPH)?;
                writer.write(ENDL)?;
                tag.render(writer)?;
                writer.write(ENDL)?;

                for node in content {
                    node.render(writer, !node.is_text())?;
                }
                writer.write(COMMA)?;
            }
            RoffNodeInner::Example(content) => {
                writer.write(ENDL)?;
                writer.write(EXAMPLE_START)?;
                writer.write(ENDL)?;
                for node in content {
                    node.render(writer)?;
                }
                writer.write(ENDL)?;
                writer.write(EXAMPLE_END)?;
                writer.write(COMMA)?;
            }
        }

        if nested {
            writer.write(ENDL)?;
            writer.write(NESTED_END)?;
        }

        Ok(())
    }
}

/// A trait that describes items that can be turned into a [`RoffNode`](RoffNode).
pub trait IntoRoffNode {
    /// Convert this item into a `RoffNode`.
    fn into_roff(self) -> RoffNode;
}

impl IntoRoffNode for RoffNodeInner {
    fn into_roff(self) -> RoffNode {
        RoffNode(self)
    }
}

impl IntoRoffNode for RoffNode {
    fn into_roff(self) -> RoffNode {
        self
    }
}

impl IntoRoffNode for RoffText {
    fn into_roff(self) -> RoffNode {
        RoffNode::text(self)
    }
}

impl IntoRoffNode for &str {
    fn into_roff(self) -> RoffNode {
        self.roff().into_roff()
    }
}

impl IntoRoffNode for String {
    fn into_roff(self) -> RoffNode {
        self.roff().into_roff()
    }
}

/// Convenience trait to convert items to [`RoffText`](RoffText).
pub trait Roffable {
    /// Returns this item as [`RoffText`](RoffText).
    fn roff(&self) -> RoffText;
}

impl Roffable for String {
    fn roff(&self) -> RoffText {
        RoffText::new(self.clone(), None)
    }
}

impl Roffable for &str {
    fn roff(&self) -> RoffText {
        self.to_string().roff()
    }
}

impl Roffable for RoffText {
    fn roff(&self) -> RoffText {
        self.clone()
    }
}

impl Roffable for u8 {
    fn roff(&self) -> RoffText {
        self.to_string().roff()
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
                        " dolor sit amet, consectetur adipiscing elit. Vivamus quis malesuada eros.".roff()
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

.P
this is some very \fBspecial\fR text
.
.SH "test section 2"

.IP "" 4
\fILorem ipsum\fR dolor sit amet, consectetur adipiscing elit\. Vivamus quis malesuada eros\.
.
.SH "test section 3"

.TP
\fBparagraph title\fR
tagged paragraph with some content
."#,
            rendered
        )
    }

    #[test]
    fn it_nests_roffs() {
        let roff = Roff::new("test", 1).add_section(
            Section::new(
                "BASE SECTION",
                vec![
                    RoffNode::paragraph(vec![
                        RoffNode::text("some text in first paragraph."),
                        RoffNode::paragraph(vec![
                            RoffNode::text("some nested paragraph"),
                            RoffNode::paragraph(vec![RoffNode::text(
                                "some doubly nested paragraph",
                            )]),
                        ]),
                    ]),
                    RoffNode::paragraph(vec!["back two levels left", " without roffs"]),
                ],
            )
            .subtitle("with some subtitle..."),
        );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH "test" "1"
.
.SH "BASE SECTION"
.SS "with some subtitle\.\.\."

.P
some text in first paragraph\.
.RS
.P
some nested paragraph
.RS
.P
some doubly nested paragraph
.
.RE
.
.RE
.
.P
back two levels left without roffs
."#,
            rendered
        )
    }

    #[test]
    fn it_roffs_examples() {
        let roff = Roff::new("test-examples", 3).section(
            "BASE SECTION",
            vec![
                RoffNode::text("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus quis malesuada eros."),
                RoffNode::example(vec![
                "let example = String::new()\n",
                "let x = example.clone();\n",
                "if x.len() > 0 {\n",
                "\tprintln!(\"{}\", x);\n",
                "}\n",
                ])
            ],
        );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH "test\-examples" "3"
.
.SH "BASE SECTION"
Lorem ipsum dolor sit amet, consectetur adipiscing elit\. Vivamus quis malesuada eros\.
.EX
let example = String::new()
let x = example\.clone();
if x\.len() > 0 {
	println!("{}", x);
}

.EE
."#,
            rendered
        )
    }

    #[test]
    fn it_escapes() {
        let input = "This-is-some-text
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus quis malesuada eros.";

        assert_eq!(
            escape(input),
            "This\\-is\\-some\\-text
Lorem ipsum dolor sit amet, consectetur adipiscing elit\\. Vivamus quis malesuada eros\\.",
        )
    }
}
