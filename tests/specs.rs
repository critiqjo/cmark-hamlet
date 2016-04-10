extern crate pulldown_cmark;
extern crate cmark_hamlet;

use std::fmt::Write;
use pulldown_cmark::Parser;
use pulldown_cmark::Options;
use cmark_hamlet::Adapter;

fn to_html(md: &str) -> String {
    let ada = Adapter::new(Parser::new_ext(md, Options::all()), false);
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
    assert_eq!(to_html("> foo\n>\n> bar"), "<blockquote><p>foo</p><p>bar</p></blockquote>");
}

#[test]
fn code_block() {
    assert_eq!(
        to_html("```\n\
                 code\n\n\
                 block\n\
                 ```"),
        "<pre><code>code\n\nblock\n</code></pre>"
    );
    assert_eq!(
        to_html("```sh\n\
                 code\n\n\
                 block\n\
                 ```"),
        "<pre data-lang=\"sh\"><code>code\n\nblock\n</code></pre>"
    );
}

#[test]
fn image() {
    assert_eq!(
        to_html("![](/link/to/image)"),
        "<p><img src=\"/link/to/image\" /></p>"
    );
    assert_eq!(
        to_html("![foo](/link/to/image)"),
        "<p><img src=\"/link/to/image\" alt=\"foo\" /></p>"
    );
    assert_eq!(
        to_html("![](/link/to/image \"bar\")"),
        "<p><img src=\"/link/to/image\" title=\"bar\" /></p>"
    );
    assert_eq!(
        to_html("![foo](/link/to/image \"bar\")"),
        "<p><img src=\"/link/to/image\" alt=\"foo\" title=\"bar\" /></p>"
    );
    assert_eq!(
        to_html("![foo] bar\n\n[foo]: /link/to/image \"bar\""),
        "<p><img src=\"/link/to/image\" alt=\"foo\" title=\"bar\" /> bar</p>"
    );
}

#[test]
fn link() {
    assert_eq!(
        to_html("<file:/link/to/x>"), // absolute URI <scheme:link>
        "<p><a href=\"file:/link/to/x\">file:/link/to/x</a></p>"
    );
    assert_eq!(
        to_html("[](/link/to/x)"),
        "<p><a href=\"/link/to/x\"></a></p>"
    );
    assert_eq!(
        to_html("[foo](/link/to/x)"),
        "<p><a href=\"/link/to/x\">foo</a></p>"
    );
    assert_eq!(
        to_html("[](/link/to/x \"bar\")"),
        "<p><a href=\"/link/to/x\" title=\"bar\"></a></p>"
    );
    assert_eq!(
        to_html("[foo](/link/to/x \"bar\")"),
        "<p><a href=\"/link/to/x\" title=\"bar\">foo</a></p>"
    );
    assert_eq!(
        to_html("[foo] bar\n\n[foo]: /link/to/x \"bar\""),
        "<p><a href=\"/link/to/x\" title=\"bar\">foo</a> bar</p>"
    );
}

#[test]
fn list() {
    assert_eq!(
        to_html("- hello\n\
                 - world"),
        "<ul><li>hello</li><li>world</li></ul>"
    );
    assert_eq!(
        to_html("1. hello\n\
                 2. world"),
        "<ol><li>hello</li><li>world</li></ol>"
    );
    assert_eq!(
        to_html("3. hello\n\
                 4. world"),
        "<ol start=\"3\"><li>hello</li><li>world</li></ol>"
    );
    assert_eq!(
        to_html("- hello\
               \n  ```\
               \n  let x;\
               \n  ```\
               \n- world"),
        "<ul><li>hello<pre><code>let x;\n</code></pre></li>\
             <li>world</li></ul>"
    );
    assert_eq!(
        to_html("- hello\n\
               \n  goo\
               \n- world"),
        "<ul><li><p>hello</p><p>goo</p></li>\
             <li><p>world</p></li></ul>" // NB: different from above!!
    );
}

#[test]
fn table() {
    assert_eq!(
        to_html("head1 | head2\n\
                 ----- | -----\n\
                 foo   | bar  "),
        "<table><tr><th>head1 </th><th> head2</th></tr>\
                <tr><td>foo   </td><td> bar</td></tr></table>"
    );
}
