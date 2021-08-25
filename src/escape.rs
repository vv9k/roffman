//!
//!
//!

pub fn escape<T: AsRef<str>>(text: T) -> String {
    let text = text.as_ref();
    let mut out = String::new();
    for token in text.chars().map(EscapeToken::from) {
        if let Some(ch) = token.unescaped_char() {
            out.push(ch);
        } else {
            out.push_str(token.escape_sequence());
        }
    }

    // Escapes dots at the beginning of the line so that they don't get interpreted as
    // roff macros.
    out.replace("\n.", "\n\\&.")
}

enum EscapeToken {
    Dash,
    LatinApostrophe,
    OpeningQuote,
    ClosingQuote,
    DoubleQuote,
    LeftDoubleQuote,
    RightDoubleQuote,
    GraveAccent,
    CircumflexAccent,
    ReverseSolidus,
    Tilde,
    Unescaped(char),
}

impl From<char> for EscapeToken {
    fn from(ch: char) -> Self {
        use EscapeToken::*;
        match ch {
            '-' => Dash,
            '\'' => LatinApostrophe,
            '‘' => OpeningQuote,
            '’' => ClosingQuote,
            '"' => DoubleQuote,
            '“' => LeftDoubleQuote,
            '”' => RightDoubleQuote,
            '`' => GraveAccent,
            '^' => CircumflexAccent,
            '\\' => ReverseSolidus,
            '~' => Tilde,
            ch => Unescaped(ch),
        }
    }
}

impl EscapeToken {
    fn escape_sequence(&self) -> &'static str {
        use EscapeToken::*;
        match self {
            Dash => "\\-",
            LatinApostrophe => "\\(aq",
            OpeningQuote => "\\(oq",
            ClosingQuote => "\\(cq",
            DoubleQuote => "\\(dq",
            LeftDoubleQuote => "\\(lq",
            RightDoubleQuote => "\\(rq",
            GraveAccent => "\\(ga",
            CircumflexAccent => "\\(ha",
            ReverseSolidus => "\\e",
            Tilde => "\\(ti",
            Unescaped(_) => "",
        }
    }

    fn unescaped_char(&self) -> Option<char> {
        if let EscapeToken::Unescaped(ch) = &self {
            Some(*ch)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn it_escapes() {
        let input = r#"~/docs/$ bash -c "awk '' ``""#;

        assert_eq!(
            escape(input),
            "\\(ti/docs/$ bash \\-c \\(dqawk \\(aq\\(aq \\(ga\\(ga\\(dq"
        );

        let dot_on_new_line = "\n.some dot on new line";

        assert_eq!(escape(dot_on_new_line), "\n\\&.some dot on new line")
    }
}
