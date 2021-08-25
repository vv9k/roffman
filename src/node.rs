use crate::_macro::*;
use crate::{write_quoted_if_whitespace, IntoRoffNode, RoffError, RoffText, Roffable, SynopsisOpt};

use std::io::Write;

#[derive(Clone, Debug)]
/// Base struct used to create ROFFs.
pub struct RoffNode(RoffNodeInner);

impl RoffNode {
    #[inline]
    pub(crate) fn into_inner(self) -> RoffNodeInner {
        self.0
    }

    #[inline]
    pub(crate) fn inner_ref(&self) -> &RoffNodeInner {
        &self.0
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
    pub fn indented_paragraph<I, R>(
        content: I,
        indentation: Option<u8>,
        title: Option<impl Roffable>,
    ) -> Self
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
            title: title.map(|t| t.roff()),
        })
    }

    /// Creates a new paragraph node with a title.
    pub fn tagged_paragraph<I, R>(content: I, title: impl Roffable) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoRoffNode,
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

    /// Nest nodes by indenting all of the nodes inside.
    pub fn nested<I, R>(nodes: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoRoffNode,
    {
        Self(RoffNodeInner::Nested(
            nodes.into_iter().map(R::into_roff).collect(),
        ))
    }

    /// Breaks the line in text. Use this instead of adding raw `\n` characters to actually render
    /// linebreaks.
    pub fn linebreak() -> Self {
        Self(RoffNodeInner::Break)
    }

    /// A long dash `—`. Used for an interruption—such as this one—in a sentence.
    pub fn em_dash() -> Self {
        Self(RoffNodeInner::EmDash)
    }

    /// A long dash `–`. Used to separate the ends of a range, particularly between number like "1–9".
    pub fn en_dash() -> Self {
        Self(RoffNodeInner::EnDash)
    }

    /// Adjustable non-breaking space.  Use this to prevent a break inside a short phrase or
    /// between a numerical quantity and its corresponding unit(s).
    pub fn non_breaking_space() -> Self {
        Self(RoffNodeInner::NonBreakingSpace)
    }

    /// Adds a comment to the generated roff. All newlines in the comment will be replaced with a ` `.
    pub fn comment<C: AsRef<str>>(comment: C) -> Self {
        Self(RoffNodeInner::Comment(comment.as_ref().to_string()))
    }
}

#[derive(Clone, Debug)]
/// Base struct used to create ROFFs.
pub(crate) enum RoffNodeInner {
    /// The most basic node type, contains only text with style.
    Text(RoffText),
    /// A simple paragraph that can contain nested items.
    Paragraph(Vec<RoffNodeInner>),
    /// Indented paragraph that can contain nested items. If no indentation is provided the default
    /// is `4`.
    IndentedParagraph {
        content: Vec<RoffNodeInner>,
        indentation: Option<u8>,
        title: Option<RoffText>,
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
    Nested(Vec<RoffNode>),
    Break,
    EmDash,
    EnDash,
    NonBreakingSpace,
    Comment(String),
}

impl RoffNodeInner {
    pub fn render<W: Write>(&self, writer: &mut W, mut was_text: bool) -> Result<bool, RoffError> {
        match self {
            RoffNodeInner::Text(text) => {
                text.render(writer)?;
                was_text = true;
            }
            RoffNodeInner::Paragraph(content) => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(PARAGRAPH)?;
                writer.write_all(ENDL)?;
                for node in content {
                    was_text = node.render(writer, was_text)?;
                }
            }
            RoffNodeInner::IndentedParagraph {
                content,
                indentation,
                title,
            } => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(INDENTED_PARAGRAPH)?;
                if let Some(indentation) = indentation {
                    writer.write_all(SPACE)?;
                    if let Some(title) = title {
                        write_quoted_if_whitespace(title, writer)?;
                    } else {
                        writer.write_all(QUOTE)?;
                        writer.write_all(QUOTE)?;
                    }
                    writer.write_all(SPACE)?;
                    indentation.roff().render(writer)?;
                }
                writer.write_all(ENDL)?;
                for node in content {
                    was_text = node.render(writer, was_text)?;
                }
                writer.write_all(ENDL)?;
                was_text = false;
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

                for node in content {
                    was_text = node.render(writer, was_text)?;
                }
                writer.write_all(ENDL)?;
                was_text = false;
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
                was_text = false;
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
                was_text = false;
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
                if !name.content().is_empty() {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(URL_END)?;
                writer.write_all(ENDL)?;
                was_text = false;
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
                if !name.content().is_empty() {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(MAIL_END)?;
                writer.write_all(ENDL)?;
                was_text = false;
            }
            RoffNodeInner::Nested(nodes) => {
                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(NESTED_START)?;
                writer.write_all(ENDL)?;
                was_text = false;
                for node in nodes {
                    was_text = node.inner_ref().render(writer, was_text)?;
                }

                if was_text {
                    writer.write_all(ENDL)?;
                }
                writer.write_all(NESTED_END)?;
                writer.write_all(ENDL)?;
                was_text = false;
            }
            RoffNodeInner::Break => {
                writer.write_all(ENDL)?;
                writer.write_all(BREAK)?;
                writer.write_all(ENDL)?;
            }
            RoffNodeInner::RegisteredSign => {
                writer.write_all(REGISTERED_SIGN)?;
                was_text = true;
            }
            RoffNodeInner::LeftQuote => {
                writer.write_all(LEFT_QUOTE)?;
                was_text = true;
            }
            RoffNodeInner::RightQuote => {
                writer.write_all(RIGHT_QUOTE)?;
                was_text = true;
            }
            RoffNodeInner::TrademarkSign => {
                writer.write_all(TRADEMARK_SIGN)?;
                was_text = true;
            }
            RoffNodeInner::EmDash => {
                writer.write_all(EM_DASH)?;
                was_text = true;
            }
            RoffNodeInner::EnDash => {
                writer.write_all(EN_DASH)?;
                was_text = true;
            }
            RoffNodeInner::NonBreakingSpace => {
                writer.write_all(NON_BREAKING_SPACE)?;
                was_text = true;
            }
            RoffNodeInner::Comment(comment) => {
                writer.write_all(COMMENT)?;
                let comment = comment.replace('\n', " ");
                writer.write_all(comment.as_bytes())?;
                writer.write_all(ENDL)?;
                was_text = false
            }
        }

        Ok(was_text)
    }
}

impl IntoRoffNode for RoffNodeInner {
    fn into_roff(self) -> RoffNode {
        RoffNode(self)
    }
}
