//! # roffman - create ROFF man pages in rust with ease!
//!
//!
//! ## Example usage
//! ```
//! use roffman::{Roff, RoffNode, Roffable, SectionNumber, SynopsisOpt};
//!
//! let roff = Roff::new("roffman", SectionNumber::Miscellaneous)
//! .date("August 2021")
//! .section(
//!    "BASIC USAGE",
//!    [
//!        RoffNode::paragraph([
//!            "This is how you create a basic paragraph using roffman.",
//!        ]),
//!        RoffNode::indented_paragraph(
//!            [
//!                "This line should be slightly indented to the ".roff(),
//!                "right.".roff().bold(),
//!            ],
//!            Some(4),
//!            Some("optional-title")
//!        ),
//!        RoffNode::synopsis(
//!                 "roffman-command",
//!                 [
//!                 "This is the description of this command. It will be displayed right next to".roff(),
//! " it".roff().italic()
//! ]                     ,
//!                 [
//!                 SynopsisOpt::new("--opt").description(["some simple opt"]),
//!                 SynopsisOpt::new("--opt-with-arg").argument("ARG").description(["opt with an argument"]),
//!                 SynopsisOpt::new("--bold")
//!        ]),
//!        RoffNode::paragraph(["Example:".roff().bold()]),
//!        RoffNode::example([
//!             r#"
//! impl Roffable for u8 {
//!     fn roff(&self) -> RoffText {
//!         self.to_string().roff()
//!     }
//! }"#,
//!         ]),
//!        RoffNode::url("GitHub", "https://github.com/vv9k/roffman"),
//!        RoffNode::text("\nvv9k"),
//!        RoffNode::trademark_sign(),
//!     ],
//! );
//!
//! let rendered = roff.to_string().unwrap();
//!
//! let output = r#".TH roffman 7 "August 2021"
//! .SH "BASIC USAGE"
//! .P
//! This is how you create a basic paragraph using roffman.
//! .IP optional\-title 4
//! This line should be slightly indented to the \fBright.\fR
//! .SY roffman\-command
//! This is the description of this command. It will be displayed right next to\fI it\fR
//!
//! .OP \-\-opt
//! some simple opt
//!
//! .OP \-\-opt\-with\-arg ARG
//! opt with an argument
//!
//! .OP \-\-bold
//!
//! .YS
//! .P
//! \fBExample:\fR
//! .EX
//!
//! impl Roffable for u8 {
//!     fn roff(&self) \-> RoffText {
//!         self.to_string().roff()
//!     }
//! }
//! .EE
//! .UR https://github.com/vv9k/roffman
//! GitHub
//! .UE
//!
//! vv9k\*(Tm
//! "#;
//!
//! assert_eq!(rendered.trim(), output.trim());
//! ```
//!
//! which will look something like this:
//! ```text
//! roffman(7)                                         Miscellaneous Information Manual                                  roffman(7)
//!
//! BASIC USAGE
//!        This is how you create a basic paragraph using roffman.
//!
//!            This line should be slightly indented to the right.
//!
//!        roffman-command This is the description of this command. It will be displayed right next to it
//!
//!                        [--opt] some simple opt
//!
//!                        [--opt-with-arg ARG] opt with an argument
//!
//!                        [--bold]
//!
//!        Example:
//!
//!        impl Roffable for u8 {
//!            fn roff(&self) -> RoffText {
//!                self.to_string().roff()
//!            }
//!        }
//!        GitHub ⟨https://github.com/vv9k/roffman⟩
//!
//!        vv9k™
//!                                                               August 2021                                             roffman(7)
//! ```

mod escape;
mod node;
mod section;
mod text;

pub use node::RoffNode;
pub use section::Section;
pub use text::{FontStyle, RoffText};

use escape::escape;

use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::io::{self, Write};

mod _macro {
    pub(crate) const SPACE: &[u8] = b" ";
    pub(crate) const QUOTE: &[u8] = b"\"";
    pub(crate) const ENDL: &[u8] = b"\n";
    pub(crate) const BOLD: &[u8] = b"\\fB";
    pub(crate) const ITALIC: &[u8] = b"\\fI";
    pub(crate) const FONT_END: &[u8] = b"\\fR";
    pub(crate) const SECTION_HEADER: &[u8] = b".SH";
    pub(crate) const SUB_HEADER: &[u8] = b".SS";
    pub(crate) const TITLE_HEADER: &[u8] = b".TH";
    pub(crate) const PARAGRAPH: &[u8] = b".P";
    pub(crate) const INDENTED_PARAGRAPH: &[u8] = b".IP";
    pub(crate) const TAGGED_PARAGRAPH: &[u8] = b".TP";
    pub(crate) const NESTED_START: &[u8] = b".RS";
    pub(crate) const NESTED_END: &[u8] = b".RE";
    pub(crate) const EXAMPLE_START: &[u8] = b".EX";
    pub(crate) const EXAMPLE_END: &[u8] = b".EE";
    pub(crate) const SYNOPSIS_START: &[u8] = b".SY";
    pub(crate) const SYNOPSIS_END: &[u8] = b".YS";
    pub(crate) const SYNOPSIS_OPT: &[u8] = b".OP";
    pub(crate) const URL_START: &[u8] = b".UR";
    pub(crate) const URL_END: &[u8] = b".UE";
    pub(crate) const MAIL_START: &[u8] = b".MT";
    pub(crate) const MAIL_END: &[u8] = b".ME";
    pub(crate) const LEFT_QUOTE: &[u8] = b"\\*(lq";
    pub(crate) const RIGHT_QUOTE: &[u8] = b"\\*(rq";
    pub(crate) const REGISTERED_SIGN: &[u8] = b"\\*R";
    pub(crate) const TRADEMARK_SIGN: &[u8] = b"\\*(Tm";
    pub(crate) const BREAK: &[u8] = b".br";
    pub(crate) const EM_DASH: &[u8] = b"\\(em";
    pub(crate) const EN_DASH: &[u8] = b"\\(en";
}
use _macro::{ENDL, QUOTE, SPACE, TITLE_HEADER};

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

fn write_quoted(roff: &RoffText, writer: &mut impl Write) -> Result<(), RoffError> {
    writer.write_all(QUOTE)?;
    roff.render(writer)?;
    writer.write_all(QUOTE)?;
    Ok(())
}

fn write_quoted_if_whitespace(roff: &RoffText, writer: &mut impl Write) -> Result<(), RoffError> {
    if roff
        .content()
        .as_bytes()
        .iter()
        .any(u8::is_ascii_whitespace)
    {
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

        let mut was_text = false;
        for section in &self.sections {
            was_text = section.render(writer, was_text)?;
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

/// A trait that describes items that can be turned into a [`RoffNode`](RoffNode).
pub trait IntoRoffNode {
    /// Convert this item into a `RoffNode`.
    fn into_roff(self) -> RoffNode;
}

impl IntoRoffNode for RoffNode {
    fn into_roff(self) -> RoffNode {
        self
    }
}

impl<R: Roffable> IntoRoffNode for R {
    fn into_roff(self) -> RoffNode {
        RoffNode::text(self.roff())
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

impl Roffable for &String {
    fn roff(&self) -> RoffText {
        RoffText::new((*self).clone(), None)
    }
}

impl Roffable for &str {
    fn roff(&self) -> RoffText {
        RoffText::new(self.to_string(), None)
    }
}

impl Roffable for &&str {
    fn roff(&self) -> RoffText {
        (*self).roff()
    }
}

impl Roffable for std::borrow::Cow<'_, str> {
    fn roff(&self) -> RoffText {
        self.as_ref().roff()
    }
}

impl Roffable for u8 {
    fn roff(&self) -> RoffText {
        RoffText::new(self.to_string(), None)
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
                [RoffNode::paragraph([
                    "this is some very ".roff(),
                    "special".roff().bold(),
                    " text".roff(),
                ])],
            )
            .section(
                "test section 2",
                [RoffNode::indented_paragraph(
                    [
                        "Lorem ipsum".roff().italic(),
                        " dolor sit amet, consectetur adipiscing elit. Vivamus quis malesuada eros.".roff()
                            .roff(),
                    ],
                    Some(4),
                    None::<&str>
                )],
            )
            .section(
                "test section 3",
                [RoffNode::tagged_paragraph(
                    ["tagged paragraph with some content".roff()],
                    "paragraph title".roff().bold(),
                )],
            )
            .section(
                "test section 4",
                [
                RoffNode::indented_paragraph(
                    [
                        "Indented paragraph with a title",
                    ],
                    Some(4),
                    Some("Paragraph title with spaces")
                ),
                RoffNode::indented_paragraph(
                    [
                        "Another indented paragraph",
                    ],
                    Some(2),
                    Some("title-no-spaces")
                )
                ],
            )
            ;

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH test 1
.SH "test section 1"
.P
this is some very \fBspecial\fR text
.SH "test section 2"
.IP "" 4
\fILorem ipsum\fR dolor sit amet, consectetur adipiscing elit. Vivamus quis malesuada eros.
.SH "test section 3"
.TP
\fBparagraph title\fR
tagged paragraph with some content
.SH "test section 4"
.IP "Paragraph title with spaces" 4
Indented paragraph with a title
.IP title\-no\-spaces 2
Another indented paragraph
"#,
            rendered
        )
    }

    #[test]
    fn it_nests_roffs() {
        let roff = Roff::new("test", SectionNumber::UserCommands).add_section(
            Section::new(
                "BASE SECTION",
                [
                    RoffNode::paragraph([
                        RoffNode::text("some text in first paragraph."),
                        RoffNode::nested([RoffNode::paragraph([
                            RoffNode::text("some nested paragraph"),
                            RoffNode::nested([RoffNode::paragraph([RoffNode::text(
                                "some doubly nested paragraph",
                            )])]),
                            RoffNode::text("some text after nested para"),
                        ])]),
                    ]),
                    RoffNode::paragraph(["back two levels left", " without roffs"]),
                ],
            )
            .subtitle("with some subtitle..."),
        );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            rendered,
            r#".TH test 1
.SH "BASE SECTION"
.SS "with some subtitle..."
.P
some text in first paragraph.
.RS
.P
some nested paragraph
.RS
.P
some doubly nested paragraph
.RE
some text after nested para
.RE
.P
back two levels left without roffs"#,
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
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus quis malesuada eros.
.EX
let example = String::new()
let x = example.clone();
if x.len() > 0 {
	println!(\(dq{}\(dq, x);
}

.EE
"#,
            rendered
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
lists files in the given\fIpath\fR.

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
                RoffNode::url("", "https://docs.rs/roffman"),
                RoffNode::url("", ""),
                RoffNode::email("John Test", "test@invalid.domain"),
                RoffNode::email("", "test@invalid.domain"),
                RoffNode::email("", ""),
            ],
        );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH test\-urls 7
.SH URLS
.UR https://github.com/vv9k/roffman
GitHub
.UE
.UR https://crates.io/crates/roffman
crates.io
.UE
.UR https://docs.rs/roffman
docs.rs
.UE
.UR https://docs.rs/roffman
.UE
.UR 
.UE
.MT test@invalid.domain
John Test
.ME
.MT test@invalid.domain
.ME
.MT 
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
                RoffNode::linebreak(),
                RoffNode::text("123"),
                RoffNode::en_dash(),
                RoffNode::text("321"),
                RoffNode::linebreak(),
                RoffNode::text("some text"),
                RoffNode::em_dash(),
                RoffNode::text("interupted sentence in the middle"),
                RoffNode::em_dash(),
                RoffNode::text("more text..."),
            ],
        );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH test\-strings 7
.SH STRINGS
\*(lqthis is some example quoted text.\*(rq \*R roffman\*(Tm
.br
123\(en321
.br
some text\(eminterupted sentence in the middle\(emmore text..."#,
            rendered
        )
    }

    #[test]
    fn section_after_text_renders() {
        let roff = Roff::new("test-sections", SectionNumber::Miscellaneous)
            .section("TEXTS", vec![RoffNode::text("this is some example text.")])
            .section(
                "NEXT",
                vec![
                    RoffNode::text("this is some example text on second section.\n"),
                    RoffNode::text("this is some example.\n"),
                    RoffNode::text("this is some example text."),
                ],
            )
            .section("THIRD", vec![RoffNode::text("this is some example text.")]);

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            r#".TH test\-sections 7
.SH TEXTS
this is some example text.
.SH NEXT
this is some example text on second section.
this is some example.
this is some example text.
.SH THIRD
this is some example text."#,
            rendered
        )
    }

    #[test]
    fn breaks_line() {
        let roff = Roff::new("test-breaks", SectionNumber::Miscellaneous).section(
            "BREAKS",
            vec![
                RoffNode::text("this is some example text."),
                RoffNode::linebreak(),
                RoffNode::text("this is some example text on second line."),
                RoffNode::linebreak(),
                RoffNode::text("this is some example text on third line."),
            ],
        );

        let rendered = roff.to_string().unwrap();
        assert_eq!(
            rendered,
            r#".TH test\-breaks 7
.SH BREAKS
this is some example text.
.br
this is some example text on second line.
.br
this is some example text on third line."#
        )
    }
}
