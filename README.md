# roffman

[![Build Status](https://github.com/vv9k/roffman/workflows/CI/badge.svg)](https://github.com/vv9k/roffman/actions?query=workflow%3A%22CI%22)


A crate to generate roff man pages.

## Usage

Add the following to the `Cargo.toml`:
```toml
[dependencies]
roffman = "0.1"
```

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
                RoffNode::paragraph(vec![
                    "And some example ".roff(),
                    "code".roff().italic(),
                    ":".roff(),
                ]),
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
    println!("{}", rendered);
}
```

Output:

```roff
.TH "roffman" "7" "August 2021"
.

.SH "BASIC USAGE"

.P
This is how you create a basic paragraph using roffman\.
.

.IP "" 4
This line should be slightly indented to the \fBright\.\fR
.

.P
And some example \fIcode\fR:
.

.EX

impl Roffable for u8 {
    fn roff(&self) \-> RoffText {
        self\.to_string()\.roff()
    }
}
.EE
```

which will look something like this:
```
roffman(7)            Miscellaneous Information Manual           roffman(7)

BASIC USAGE
       This is how you create a basic paragraph using roffman.

           This line should be slightly indented to the right.

       And some example code:

       impl Roffable for u8 {
           fn roff(&self) -> RoffText {
               self.to_string().roff()
           }
       }

                                                                 roffman(7)
```

## License
[MIT](https://github.com/vv9k/roffman/blob/master/LICENSE)
