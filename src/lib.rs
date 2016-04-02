extern crate pulldown_cmark as cmark;
#[macro_use]
extern crate hamlet;

mod adapter;

pub use adapter::Adapter;

#[cfg(test)]
mod tests {
    use hamlet::HtmlWriter;
    use cmark::Parser;
    use Adapter;

    #[test]
    fn full_stack() {
        let md = "Ok, [google][ref]\n\n- - -\n\n\
                  ```rust\n\
                  use hamlet;\n\
                  ```\n\n\
                  [ref]: http://google.com";
        let ada = Adapter::new(Parser::new(md), false);
        let mut result = Vec::new();
        HtmlWriter::new(ada).write_to(&mut result).unwrap();
        let res_str = String::from_utf8(result).unwrap();
        assert_eq!("<p>Ok, <a href=\"http://google.com\">google</a></p><hr />\
                   <pre data-lang=\"rust\"><code>use hamlet;\n</code></pre>",
                   res_str);
    }
}
