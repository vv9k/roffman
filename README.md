# roffman

[![Build Status](https://github.com/vv9k/roffman/workflows/CI/badge.svg)](https://github.com/vv9k/roffman/actions?query=workflow%3A%22CI%22)


A crate to generate roff man pages.

## Example
```rust
use roffman::{IntoRoffNode, Roff, Roffable, RoffNode};

fn main() {
    let roff = Roff::new("roffman-manual", 7)
    .date("August 2021")
    .section(
        "BASIC USAGE",
        vec![
            RoffNode::paragraph(vec![
                "This is how you create a basic paragraph using roffman.",
            ]),
            RoffNode::indented_paragraph(
                vec![
                    "This line should be slightly indented to the ".roff(),
                    "right.".roff().bold(),
                ],
                Some(4),
            ),
            RoffNode::text("And some example "),
            RoffNode::text("code".roff().italic()),
            RoffNode::text(":"),
            RoffNode::example(vec![
                r#"
impl Roffable for u8 {
    fn roff(&self) -> RoffText {
        self.to_string().roff()
    }
}"#,
            ]),
        ],
    );

    let rendered = roff.to_string().unwrap();
    
    let output = r#"
.TH "roffman" "7"
.
.SH "BASIC USAGE"

.P
This is how you create a basic paragraph using roffman\.
.br

.IP "" 4
This line should be slightly indented to the \fBright\.\fR
.br
And some example \fIcode\fR:
.EX

impl Roffable for u8 {
    fn roff(&self) \-> RoffText {
        self\.to_string()\.roff()
    }
}
.EE"#;

    assert_eq!(rendered.trim(), output.trim());
}
```

Output:

```roff
```

## License
[MIT](https://github.com/vv9k/roffman/blob/master/LICENSE)
