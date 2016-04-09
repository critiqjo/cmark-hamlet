//! This library provides an adapter from [pulldown-cmark][] events to
//! [hamlet][] tokens.
//!
//! [pulldown-cmark]: https://github.com/google/pulldown-cmark
//! [hamlet]: https://github.com/Nemo157/hamlet
//!
//! ## Example
//!
//! ```rust
//! extern crate pulldown_cmark;
//! extern crate cmark_hamlet;
//!
//! use std::fmt::Write;
//! use pulldown_cmark::Parser;
//! use cmark_hamlet::Adapter;
//!
//! fn main() {
//!     let md = "Ok, [google][ref]\n\n- - -\n\n\
//!               ```rust\n\
//!               use hamlet;\n\
//!               ```\n\n\
//!               [ref]: http://google.com";
//!     let ada = Adapter::new(Parser::new(md), false);
//!     let mut res = String::from("");
//!     for token in ada {
//!         write!(res, "{}", token).unwrap();
//!     }
//!     assert_eq!("<p>Ok, <a href=\"http://google.com\">google</a></p><hr />\
//!                <pre data-lang=\"rust\"><code>use hamlet;\n</code></pre>",
//!                res);
//! }
//! ```
//!
//! ## Translation
//!
//! ### Event mapping
//!
//! `pulldown_cmark` event | Action
//! ---------------------- | ------
//! `Start(tag)`           | Context dependent; see [Tag mapping](#tag-mapping)
//! `End(tag)`             | Context dependent
//! `Text(text)`           | Context dependent; see [Text handling](#text-handling)
//! `Html(html)`           | `Token::RawText(html)`
//! `InlineHtml(html)`     | `Token::RawText(html)`
//! `SoftBreak`            | `"\n"` (see [Text handling](#text-handling))
//! `HardBreak`            | `Token` representing `<br />`
//! `FootnoteReference(_)` | unimplemented!
//!
//! ### Tag mapping
//!
//! `pulldown_cmark` tag    | Html tag name
//! ----------------------- | -------------
//! `BlockQuote`            | `blockquote`
//! `CodeBlock(lang)`       | `pre` and `code` (see [`CodeBlock` handling](#codeblock-handling))
//! `Code`                  | `code`
//! `Emphasis`              | `em`
//! `FootnoteDefinition(_)` | unimplemented!
//! `Header(level)`         | `h{level}`
//! `Image(_, _)`           | `img`
//! `Item`                  | `li`
//! `Link(_, _)`            | `a`
//! `List(None)`            | `ul`
//! `List(Some(_))`         | `ol`
//! `Paragraph`             | `p`
//! `Rule`                  | `hr`
//! `Strong`                | `strong`
//! `Table(_)`              | `table`
//! `TableCell`             | `td`
//! `TableHead`             | `tr`
//! `TableRow`              | `tr`
//!
//! ### Text handling
//!
//! All successive `Text` and `SoftBreak` events are clumped together to form a
//! single `hamlet::Text` token, if [`group_text`
//! argument](struct.Adapter.html#method.new) provided was `true`.
//!
//! ### `CodeBlock` handling
//!
//! If `lang` attribute is not empty, then a `data-lang` attribute is added to
//! the `pre` tag with that value.

#![warn(missing_docs)]

extern crate pulldown_cmark as cmark;
#[macro_use]
extern crate hamlet;

mod adapter;

pub use adapter::Adapter;
