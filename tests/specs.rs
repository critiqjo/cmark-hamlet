extern crate pulldown_cmark;
extern crate cmark_hamlet;

use std::fmt::Write;
use pulldown_cmark::Parser;
use cmark_hamlet::Adapter;

fn to_html(md: &str) -> String {
    let ada = Adapter::new(Parser::new(md), false);
    let mut res = String::from("");
    for token in ada {
        write!(res, "{}", token).unwrap();
    }
    res
}

#[test]
fn simple() {
    assert_eq!(to_html("foo  bar"), "<p>foo  bar</p>");
    assert_eq!(to_html("foo\nbar"), "<p>foo\nbar</p>");
    assert_eq!(to_html("foo\n\nbar"), "<p>foo</p><p>bar</p>");
    assert_eq!(to_html("foo\n\nbar"), "<p>foo</p><p>bar</p>");
    assert_eq!(to_html("foo\n- - -\nbar"), "<p>foo</p><hr /><p>bar</p>");
    assert_eq!(to_html("`foo bar`"), "<p><code>foo bar</code></p>");
    assert_eq!(to_html("**foo bar**"), "<p><strong>foo bar</strong></p>");
    assert_eq!(to_html("_foo bar_"), "<p><em>foo bar</em></p>");
}
