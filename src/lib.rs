//! # roffman - create ROFF man pages in rust with ease!
//!
//!
//! ## Example usage
//! ```
//! use roffman::{IntoRoffNode, Roff, Roffable, RoffNode, SectionNumber};
//!
//! let roff = Roff::new("roffman", SectionNumber::Miscellaneous)
//!     .date("August 2021")
//!     .section(
//!        "BASIC USAGE",
//!        vec![
//!            RoffNode::paragraph(vec![
//!                "This is how you create a basic paragraph using roffman.",
//!            ]),
//!            RoffNode::indented_paragraph(
//!                vec![
//!                    "This line should be slightly indented to the ".roff(),
//!                    "right.".roff().bold(),
//!                ],
//!                Some(4),
//!            ),
//!            RoffNode::paragraph(vec![
//!                "And some example ".roff(),
//!                "code".roff().italic(),
//!                ":".roff(),
//!            ]),
//!            RoffNode::example(vec![
//!                r#"
//! impl Roffable for u8 {
//!     fn roff(&self) -> RoffText {
//!         self.to_string().roff()
//!     }
//! }"#,
//!             ]),
//!         ],
//!     );
//!
//! let rendered = roff.to_string().unwrap();
//!
//! let output = r#"
//! .TH roffman 7 "August 2021"
//! .SH "BASIC USAGE"
//! .P
//! This is how you create a basic paragraph using roffman\.
//! .IP "" 4
//! This line should be slightly indented to the \fBright\.\fR
//! .P
//! And some example \fIcode\fR:
//! .EX
//!
//! impl Roffable for u8 {
//!     fn roff(&self) \-> RoffText {
//!         self\.to_string()\.roff()
//!     }
//! }
//! .EE
//! "#;
//!
//! assert_eq!(rendered.trim(), output.trim());
//! ```
//!
//!
//! which will look something like this:
//! ```text
//! roffman(7)                      Miscellaneous Information Manual                     roffman(7)
//!
//! BASIC USAGE
//!        This is how you create a basic paragraph using roffman.
//!
//!            This line should be slightly indented to the right.
//!
//!        And some example code:
//!
//!        impl Roffable for u8 {
//!            fn roff(&self) -> RoffText {
//!                self.to_string().roff()
//!            }
//!        }
//!
//!                                           August 2021                                roffman(7)
//! ```

use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::io::{self, Write};

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
const SYNOPSIS_START: &[u8] = b".SY";
const SYNOPSIS_END: &[u8] = b".YS";
const SYNOPSIS_OPT: &[u8] = b".OP";
const URL_START: &[u8] = b".UR";
const URL_END: &[u8] = b".UE";
const MAIL_START: &[u8] = b".MT";
const MAIL_END: &[u8] = b".ME";
const LEFT_QUOTE: &[u8] = b"\\*(lq";
const RIGHT_QUOTE: &[u8] = b"\\*(rq";
const REGISTERED_SIGN: &[u8] = b"\\*R";
const TRADEMARK_SIGN: &[u8] = b"\\*(Tm";

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
    writer.write_all(QUOTE)?;
    roff.render(writer)?;
    writer.write_all(QUOTE)?;
    Ok(())
}

fn write_quoted_if_whitespace(roff: &RoffText, writer: &mut impl Write) -> Result<(), RoffError> {
    if roff.content.as_bytes().iter().any(u8::is_ascii_whitespace) {
        write_quoted(roff, writer)
    } else {
        roff.render(writer)
    }
}

#[derive(Clone, Debug)]
/// Represents a ROFF document that can be rendered and displayed
/// with tools like [`man`](https://man7.org/linux/man-pages/man1/man.1.html).
pub struct Roff {
    title: RoffText,
    date: Option<RoffText>,
    section: SectionNumber,
    sections: Vec<Section>,
}

impl Roff {
    /// Create a new `Roff` with a `title` and a `section`.
    pub fn new(title: impl Roffable, section: SectionNumber) -> Self {
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
        writer.write_all(SPACE)?;
        write_quoted_if_whitespace(&self.title, writer)
    }

    fn write_section(&self, writer: &mut impl Write) -> Result<(), RoffError> {
        writer.write_all(SPACE)?;
        write_quoted_if_whitespace(&self.section.roff(), writer)
    }

    fn write_date(&self, writer: &mut impl Write) -> Result<(), RoffError> {
        if let Some(date) = &self.date {
            writer.write_all(SPACE)?;
            write_quoted_if_whitespace(date, writer)?;
        }
        Ok(())
    }

    fn write_title_header(&self, writer: &mut impl Write) -> Result<(), RoffError> {
        writer.write_all(TITLE_HEADER)?;
        self.write_title(writer)?;
        self.write_section(writer)?;
        self.write_date(writer)?;
        writer.write_all(ENDL)?;
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

#[derive(Copy, Clone, Debug, PartialEq)]
/// Defines the section to which the given ROFF belongs.
pub enum SectionNumber {
    ///Commands that can be executed by the user from within a shell.
    UserCommands,
    /// Functions which wrap operations performed by the kernel.
    SystemCalls,
    /// All library functions excluding the system call wrappers (Most of the libc functions).
    LibraryCalls,
    /// Files found in `/dev` which allow to access to devices through the kernel.
    Devices,
    /// Describes various human-readable file formats and configuration files.
    FileFormatsAndConfigurationFiles,
    /// Games and funny little programs available on the system.
    Games,
    /// Overviews or descriptions of various topics, conventions, and protocols, character set
    /// standards, the standard filesystem layout, and miscellaneous other things.
    Miscellaneous,
    /// Commands like `mount(8)`, many of which only root can execute.
    SystemManagementCommands,
    /// A custom section number.
    Custom(u8),
}

impl From<SectionNumber> for u8 {
    fn from(s: SectionNumber) -> Self {
        use SectionNumber::*;
        match s {
            UserCommands => 1,
            SystemCalls => 2,
            LibraryCalls => 3,
            Devices => 4,
            FileFormatsAndConfigurationFiles => 5,
            Games => 6,
            Miscellaneous => 7,
            SystemManagementCommands => 8,
            Custom(n) => n,
        }
    }
}

impl Roffable for SectionNumber {
    fn roff(&self) -> RoffText {
        u8::from(*self).roff()
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
            node.render(writer, false, was_text)?;
            was_text = node.is_text();
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Style that can be applied to [`RoffText`](RoffText)
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
/// Wrapper type for styled text in ROFF.
pub struct RoffText {
    content: String,
    style: FontStyle,
}

impl RoffText {
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

    fn render<W: Write>(&self, writer: &mut W) -> Result<(), RoffError> {
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

#[derive(Clone, Debug)]
/// Base struct used to create ROFFs.
pub struct RoffNode(RoffNodeInner);

impl RoffNode {
    #[inline]
    fn into_inner(self) -> RoffNodeInner {
        self.0
    }

    /// Creates a simple text node.
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

    /// Creates a new example node. An example block usually has the font set to monospaced.
    pub fn example<I, R>(content: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Roffable,
    {
        Self(RoffNodeInner::Example(
            content.into_iter().map(|item| item.roff()).collect(),
        ))
    }

    /// Creates a new synopsis node explaining the given `command` with `description` and `opts`.
    pub fn synopsis<I, R, O>(command: impl Roffable, description: I, opts: O) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Roffable,
        O: IntoIterator<Item = SynopsisOpt>,
    {
        Self(RoffNodeInner::Synopsis {
            command: command.roff(),
            text: description.into_iter().map(|item| item.roff()).collect(),
            opts: opts.into_iter().collect(),
        })
    }

    /// Creates a new URL node that will take the form of `[name](address)` where `name` is the
    /// visible part of the URL and address is where it points to.
    pub fn url(name: impl Roffable, address: impl Roffable) -> Self {
        Self(RoffNodeInner::Url {
            name: name.roff(),
            address: address.roff(),
        })
    }

    /// Creates a new email node that will where `address` is the email address and `name` is the
    /// visible URL text. `address` may not be visible if the man page is being viewed as HTML.
    pub fn email(name: impl Roffable, address: impl Roffable) -> Self {
        Self(RoffNodeInner::Email {
            name: name.roff(),
            address: address.roff(),
        })
    }

    /// Returns a node that will be rendered as a registered sign `®`.
    pub fn registered_sign() -> Self {
        Self(RoffNodeInner::RegisteredSign)
    }

    /// Returns a node that will be rendered as a left quote `“`.
    pub fn left_quote() -> Self {
        Self(RoffNodeInner::LeftQuote)
    }

    /// Returns a node that will be rendered as a right quote `”`.
    pub fn right_quote() -> Self {
        Self(RoffNodeInner::RightQuote)
    }

    /// Returns a node that will be rendered as a trademark sign `™`.
    pub fn trademark_sign() -> Self {
        Self(RoffNodeInner::TrademarkSign)
    }
}

#[derive(Clone, Debug)]
/// An option used by the [`RoffNode::synopsis`](RoffNode::synopsis) block.
pub struct SynopsisOpt {
    name: RoffText,
    argument: Option<RoffText>,
    description: Option<Vec<RoffText>>,
}

impl SynopsisOpt {
    /// Creates a new option used in a synopsis block.
    pub fn new<R: Roffable>(name: R) -> Self {
        Self {
            name: name.roff(),
            argument: None,
            description: None,
        }
    }

    /// Set the name of the argument that this option takes.
    pub fn argument<R: Roffable>(mut self, argument: R) -> Self {
        self.argument = Some(argument.roff());
        self
    }

    /// Set the description for this command synopsis.
    pub fn description<I, R>(mut self, description: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Roffable,
    {
        self.description = Some(description.into_iter().map(|item| item.roff()).collect());
        self
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
    Synopsis {
        command: RoffText,
        text: Vec<RoffText>,
        opts: Vec<SynopsisOpt>,
    },
    Url {
        name: RoffText,
        address: RoffText,
    },
    Email {
        name: RoffText,
        address: RoffText,
    },
    RegisteredSign,
    LeftQuote,
    RightQuote,
    TrademarkSign,
}

impl RoffNodeInner {
    /// Returns `true` if the node is the [`RoffNode::Text`](RoffNode::Text) variant.
    fn is_text(&self) -> bool {
        matches!(self, &RoffNodeInner::Text(_))
    }

    fn is_nestable(&self) -> bool {
        !self.is_text()
    }

    fn render<W: Write>(
        &self,
        writer: &mut W,
        nested: bool,
        was_text: bool,
    ) -> Result<(), RoffError> {
        if nested {
            writer.write_all(ENDL)?;
            writer.write_all(NESTED_START)?;
        }
        match self {
            RoffNodeInner::Text(text) => {
                let styled = match text.style {
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

                writer.write_all(text.content.as_bytes())?;
                if styled {
                    writer.write_all(FONT_END)?;
                }
            }
            RoffNodeInner::Paragraph(content) => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(PARAGRAPH)?;
                writer.write_all(ENDL)?;
                let mut was_text_node = false;
                for node in content {
                    node.render(writer, node.is_nestable(), was_text_node)?;
                    was_text_node = node.is_text();
                }
                writer.write_all(ENDL)?;
            }
            RoffNodeInner::IndentedParagraph {
                content,
                indentation,
            } => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(INDENTED_PARAGRAPH)?;
                if let Some(indentation) = indentation {
                    writer.write_all(format!(" \"\" {}", indentation).as_bytes())?;
                }
                writer.write_all(ENDL)?;
                let mut was_text_node = false;
                for node in content {
                    node.render(writer, node.is_nestable(), was_text_node)?;
                    was_text_node = node.is_text();
                }
                writer.write_all(ENDL)?;
            }
            RoffNodeInner::TaggedParagraph {
                content,
                title: tag,
            } => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(TAGGED_PARAGRAPH)?;
                writer.write_all(ENDL)?;
                tag.render(writer)?;
                writer.write_all(ENDL)?;

                let mut was_text_node = false;
                for node in content {
                    node.render(writer, node.is_nestable(), was_text_node)?;
                    was_text_node = node.is_text();
                }
                writer.write_all(ENDL)?;
            }
            RoffNodeInner::Example(content) => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(EXAMPLE_START)?;
                writer.write_all(ENDL)?;
                for node in content {
                    node.render(writer)?;
                }
                writer.write_all(ENDL)?;
                writer.write_all(EXAMPLE_END)?;
                writer.write_all(ENDL)?;
            }
            RoffNodeInner::Synopsis {
                command,
                text,
                opts,
            } => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(SYNOPSIS_START)?;
                writer.write_all(SPACE)?;
                write_quoted_if_whitespace(command, writer)?;
                writer.write_all(ENDL)?;
                for elem in text {
                    elem.render(writer)?;
                }
                if !text.is_empty() {
                    writer.write_all(ENDL)?;
                }
                for op in opts {
                    writer.write_all(ENDL)?;
                    writer.write_all(SYNOPSIS_OPT)?;
                    writer.write_all(SPACE)?;
                    write_quoted_if_whitespace(&op.name, writer)?;
                    if let Some(arg) = &op.argument {
                        writer.write_all(SPACE)?;
                        write_quoted_if_whitespace(arg, writer)?;
                    }
                    writer.write_all(ENDL)?;
                    if let Some(description) = &op.description {
                        for elem in description {
                            elem.render(writer)?;
                        }
                    }
                    writer.write_all(ENDL)?;
                }
                writer.write_all(SYNOPSIS_END)?;
                writer.write_all(ENDL)?;
            }
            RoffNodeInner::Url { address, name } => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(URL_START)?;
                writer.write_all(SPACE)?;
                address.render(writer)?;
                writer.write_all(ENDL)?;
                name.render(writer)?;
                writer.write_all(ENDL)?;
                writer.write_all(URL_END)?;
                writer.write_all(ENDL)?;
            }
            RoffNodeInner::Email { address, name } => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(MAIL_START)?;
                writer.write_all(SPACE)?;
                address.render(writer)?;
                writer.write_all(ENDL)?;
                name.render(writer)?;
                writer.write_all(ENDL)?;
                writer.write_all(MAIL_END)?;
                writer.write_all(ENDL)?;
            }
            RoffNodeInner::RegisteredSign => writer.write_all(REGISTERED_SIGN)?,
            RoffNodeInner::LeftQuote => writer.write_all(LEFT_QUOTE)?,
            RoffNodeInner::RightQuote => writer.write_all(RIGHT_QUOTE)?,
            RoffNodeInner::TrademarkSign => writer.write_all(TRADEMARK_SIGN)?,
        }

        if nested {
            writer.write_all(ENDL)?;
            writer.write_all(NESTED_END)?;
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
        let roff = Roff::new("test", SectionNumber::UserCommands)
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
            r#".TH test 1
.SH "test section 1"
.P
this is some very \fBspecial\fR text
.SH "test section 2"
.IP "" 4
\fILorem ipsum\fR dolor sit amet, consectetur adipiscing elit\. Vivamus quis malesuada eros\.
.SH "test section 3"
.TP
\fBparagraph title\fR
tagged paragraph with some content
"#,
            rendered
        )
    }

    #[test]
    fn it_nests_roffs() {
        let roff = Roff::new("test", SectionNumber::UserCommands).add_section(
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
            r#".TH test 1
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

.RE

.RE
.P
back two levels left without roffs
"#,
            rendered
        )
    }

    #[test]
    fn it_roffs_examples() {
        let roff = Roff::new("test-examples", SectionNumber::LibraryCalls).section(
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
            r#".TH test\-examples 3
.SH "BASE SECTION"
Lorem ipsum dolor sit amet, consectetur adipiscing elit\. Vivamus quis malesuada eros\.
.EX
let example = String::new()
let x = example\.clone();
if x\.len() > 0 {
	println!("{}", x);
}

.EE
"#,
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

    #[test]
    fn synopsis_works() {
        let roff = Roff::new("test-synopsis", SectionNumber::Miscellaneous).section(
            "SYNOPSIS",
            vec![
                RoffNode::synopsis("ls", ["lists files in the given".roff(), "path".roff().italic(), ".".roff()],
                vec![
                    SynopsisOpt::new("-l").description(["use a long listing format"]),
                    SynopsisOpt::new("-L, --dereference").description(["when showing file information for a symbolic link, show information for the file the link references rather than for the link itself"]),
                    SynopsisOpt::new("--block-size").argument("SIZE").description(["with -l, scale sizes by SIZE when printing them"]),
                ]
                )
            ],
        );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH test\-synopsis 7
.SH SYNOPSIS
.SY ls
lists files in the given\fIpath\fR\.

.OP \-l
use a long listing format

.OP "\-L, \-\-dereference"
when showing file information for a symbolic link, show information for the file the link references rather than for the link itself

.OP \-\-block\-size SIZE
with \-l, scale sizes by SIZE when printing them
.YS
"#,
            rendered
        )
    }

    #[test]
    fn urls_and_emails_work() {
        let roff = Roff::new("test-urls", SectionNumber::Miscellaneous).section(
            "URLS",
            vec![
                RoffNode::url("GitHub", "https://github.com/vv9k/roffman"),
                RoffNode::url("crates.io", "https://crates.io/crates/roffman"),
                RoffNode::url("docs.rs", "https://docs.rs/roffman"),
                RoffNode::email("John Test", "test@invalid.domain"),
            ],
        );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH test\-urls 7
.SH URLS
.UR https://github\.com/vv9k/roffman
GitHub
.UE
.UR https://crates\.io/crates/roffman
crates\.io
.UE
.UR https://docs\.rs/roffman
docs\.rs
.UE
.MT test@invalid\.domain
John Test
.ME
"#,
            rendered
        )
    }

    #[test]
    fn special_strings_work() {
        let roff = Roff::new("test-strings", SectionNumber::Miscellaneous).section(
            "STRINGS",
            vec![
                RoffNode::left_quote(),
                RoffNode::text("this is some example quoted text."),
                RoffNode::right_quote(),
                RoffNode::text(" "),
                RoffNode::registered_sign(),
                RoffNode::text(" roffman"),
                RoffNode::trademark_sign(),
            ],
        );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH test\-strings 7
.SH STRINGS
\*(lqthis is some example quoted text\.\*(rq \*R roffman\*(Tm"#,
            rendered
        )
    }
}
