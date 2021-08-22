# roffman

[![Build Status](https://github.com/vv9k/roffman/workflows/CI/badge.svg)](https://github.com/vv9k/roffman/actions?query=workflow%3A%22CI%22)
[![Docs](https://docs.rs/roffman/badge.svg)](https://docs.rs/roffman)



A crate to generate roff man pages.

## Usage

Add the following to the `Cargo.toml`:
```toml
[dependencies]
roffman = "0.2"
```

## Example
```rust
use roffman::{Roff, RoffNode, RoffNode, Roffable, SectionNumber, SynopsisOpt};

fn main() {
    let roff = Roff::new("roffman", SectionNumber::Miscellaneous)
    .date("August 2021")
    .section(
       "BASIC USAGE",
       [
           RoffNode::paragraph([
               "This is how you create a basic paragraph using roffman.",
           ]),
           RoffNode::indented_paragraph(
               [
                   "This line should be slightly indented to the ".roff(),
                   "right.".roff().bold(),
               ],
               Some(4),
               Some("optional-title")
           ),
           RoffNode::synopsis(
                    "roffman-command",
                    [
                    "This is the description of this command. It will be displayed right next to".roff(), " it".roff().italic()
]                     ,
                    [
                    SynopsisOpt::new("--opt").description(["some simple opt"]),
                    SynopsisOpt::new("--opt-with-arg").argument("ARG").description(["opt with an argument"]),
                    SynopsisOpt::new("--bold")
           ]),
           RoffNode::paragraph(["Example:".roff().bold()]),
           RoffNode::example([
               r#"
impl Roffable for u8 {
    fn roff(&self) -> RoffText {
        self.to_string().roff()
    }
}"#,
            ]),
           RoffNode::url("GitHub", "https://github.com/vv9k/roffman"),
           RoffNode::text("\nvv9k"),
           RoffNode::trademark_sign(),
            
        ],
    );

    let rendered = roff.to_string().unwrap();
    println!("{}", rendered);

}

```

Output:
```roff
.TH roffman 7 "August 2021"
.SH "BASIC USAGE"
.P
This is how you create a basic paragraph using roffman\.
.IP optional\-title 4
This line should be slightly indented to the \fBright\.\fR
.SY roffman\-command
This is the description of this command\. It will be displayed right next to \fIit\fR

.OP \-\-opt
some simple opt

.OP \-\-opt\-with\-arg ARG
opt with an argument

.OP \-\-bold

.YS
.P
\fBExample:\fR
.EX

impl Roffable for u8 {
    fn roff(&self) \-> RoffText {
        self\.to_string()\.roff()
    }
}
.EE
.UR https://github\.com/vv9k/roffman
GitHub
.UE

vv9k\*(Tm
```

which will look something like this:
```
roffman(7)                                         Miscellaneous Information Manual                                  roffman(7)

BASIC USAGE
       This is how you create a basic paragraph using roffman.

           This line should be slightly indented to the right.

       roffman-command This is the description of this command. It will be displayed right next to it

                       [--opt] some simple opt

                       [--opt-with-arg ARG] opt with an argument

                       [--bold]

       Example:

       impl Roffable for u8 {
           fn roff(&self) -> RoffText {
               self.to_string().roff()
           }
       }
       GitHub ⟨https://github.com/vv9k/roffman⟩

       vv9k™

                                                              August 2021                                             roffman(7)
```

## License
[MIT](https://github.com/vv9k/roffman/blob/master/LICENSE)
