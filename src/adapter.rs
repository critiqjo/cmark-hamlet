use std::borrow::Cow;

use cmark::Tag as CmTag;
use cmark::Event as CmEvent;
use hamlet::Token as HmToken;

/// _The_ adapter! An iterator that generates `hamlet::Token`s.
pub struct Adapter<'a, I> {
    cm_iter: I,
    cm_looka: Option<CmEvent<'a>>, // for lookahead
    hm_queue: Vec<HmToken<'a>>,
    group_text: bool,
    table_head: bool,
}

impl<'a, I> Adapter<'a, I>
    where I: Iterator<Item = CmEvent<'a>>
{
    /// Create a `Token` iterator from an `Event` iterable. See [Text
    /// handling](index.html#text-handling) for more details.
    pub fn new<T>(iterable: T, group_text: bool) -> Adapter<'a, I>
        where T: IntoIterator<IntoIter = I, Item = CmEvent<'a>> + 'a
    {
        Adapter {
            cm_iter: iterable.into_iter(),
            cm_looka: None,
            hm_queue: Vec::with_capacity(2),
            group_text: group_text,
            table_head: false,
        }
    }

    fn cm_start_tag(&mut self, tag: CmTag<'a>) -> HmToken<'a> {
        match tag {
            CmTag::Rule => {
                let _ = self.cm_iter.next(); // skip End(Rule)
                HmToken::start_tag("hr", attrs!()).closed()
            }
            CmTag::Code |
            CmTag::Strong |
            CmTag::Emphasis |
            CmTag::Paragraph |
            CmTag::BlockQuote |
            CmTag::Table(_) |
            CmTag::TableRow |
            CmTag::TableCell |
            CmTag::Item |
            CmTag::List(None) |
            CmTag::List(Some(1)) |
            CmTag::Header(_) => HmToken::start_tag(self.tag_map(tag), attrs!()),
            CmTag::TableHead => {
                self.table_head = true;
                HmToken::start_tag("tr", attrs!())
            }
            CmTag::List(Some(start)) => {
                HmToken::start_tag("ol", attrs!(start = format!("{}", start)))
            }
            CmTag::CodeBlock(lang) => {
                self.hm_queue.push(HmToken::start_tag("code", attrs!()));
                if lang.is_empty() {
                    HmToken::start_tag("pre", attrs!())
                } else {
                    HmToken::start_tag("pre", attrs!(dataLang = lang))
                }
            }
            CmTag::Image(src, title) => {
                let mut alt = String::from("");
                while let Some(cm_ev) = self.cm_iter.next() {
                    match cm_ev {
                        CmEvent::Text(text) => alt.push_str(text.as_ref()),
                        CmEvent::End(CmTag::Image(_, _)) => break,
                        CmEvent::Start(CmTag::Image(_, _)) => unreachable!(),
                        _ => (), // ignore other events
                    }
                }
                let mut attrs = attrs!(src = src);
                if !alt.is_empty() {
                    attrs.set("alt", alt);
                }
                if !title.is_empty() {
                    attrs.set("title", title);
                }
                HmToken::start_tag("img", attrs).closed()
            }
            CmTag::Link(href, title) => {
                let mut attrs = attrs!(href = href);
                if !title.is_empty() {
                    attrs.set("title", title);
                }
                HmToken::start_tag("a", attrs)
            }
            CmTag::FootnoteDefinition(_) => unimplemented!(),
        }
    }

    fn cm_end_tag(&mut self, tag: CmTag<'a>) -> HmToken<'a> {
        match tag {
            CmTag::Rule => unreachable!(),
            CmTag::Code |
            CmTag::Strong |
            CmTag::Emphasis |
            CmTag::Paragraph |
            CmTag::BlockQuote |
            CmTag::Table(_) |
            CmTag::TableRow |
            CmTag::TableCell |
            CmTag::Item |
            CmTag::List(_) |
            CmTag::Header(_) => HmToken::end_tag(self.tag_map(tag)),
            CmTag::TableHead => {
                self.table_head = false;
                HmToken::end_tag("tr")
            }
            CmTag::CodeBlock(_) => {
                self.hm_queue.push(HmToken::end_tag("pre"));
                HmToken::end_tag("code")
            }
            CmTag::Image(_, _) => unreachable!(),
            CmTag::Link(_, _) => HmToken::end_tag("a"),
            CmTag::FootnoteDefinition(_) => unimplemented!(),
        }
    }

    fn cm_text(&mut self, mut s: Cow<'a, str>) -> HmToken<'a> {
        if self.group_text {
            while let Some(cm_ev) = self.cm_iter.next() {
                match cm_ev {
                    CmEvent::Text(text) => s.to_mut().push_str(text.as_ref()),
                    CmEvent::SoftBreak => s.to_mut().push_str("\n"),
                    _ => {
                        self.cm_looka = Some(cm_ev);
                        break;
                    }
                }
            }
        }
        HmToken::Text(s)
    }

    fn tag_map(&self, tag: CmTag<'a>) -> Cow<'a, str> {
        match tag {
            CmTag::Rule => "hr".into(),
            CmTag::Code => "code".into(),
            CmTag::Strong => "strong".into(),
            CmTag::Emphasis => "em".into(),
            CmTag::Paragraph => "p".into(),
            CmTag::BlockQuote => "blockquote".into(),
            CmTag::Table(_) => "table".into(),
            CmTag::TableRow => "tr".into(),
            CmTag::TableCell => {
                if self.table_head {
                    "th".into()
                } else {
                    "td".into()
                }
            }
            CmTag::Item => "li".into(),
            CmTag::List(None) => "ul".into(),
            CmTag::List(Some(_)) => "ol".into(),
            CmTag::Header(level) => format!("h{}", level).into(),
            _ => unreachable!(),
        }
    }
}

impl<'a, I> Iterator for Adapter<'a, I>
    where I: Iterator<Item = CmEvent<'a>>
{
    type Item = HmToken<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.hm_queue.is_empty() {
            Some(self.hm_queue.remove(0))
        } else {
            let cm_ev = if let Some(cm_ev) = self.cm_looka.take() {
                cm_ev
            } else if let Some(cm_ev) = self.cm_iter.next() {
                cm_ev
            } else {
                return None;
            };
            let hm_ev = match cm_ev {
                CmEvent::Start(tag) => self.cm_start_tag(tag),
                CmEvent::End(tag) => self.cm_end_tag(tag),
                CmEvent::Text(text) => self.cm_text(text),
                CmEvent::Html(html) | CmEvent::InlineHtml(html) => HmToken::RawText(html),
                CmEvent::SoftBreak => self.cm_text("\n".into()),
                CmEvent::HardBreak => HmToken::start_tag("br", attrs!()).closed(),
                CmEvent::FootnoteReference(_) => unimplemented!(),
            };
            Some(hm_ev)
        }
    }
}

#[cfg(test)]
mod tests {
    use hamlet::Token as HmToken;
    use cmark::Event as CmEvent;
    use cmark::Parser;
    use Adapter;
    use std::borrow::Cow;

    fn html_skel_map<'a, I>(ada: Adapter<'a, I>) -> Vec<Cow<'a, str>>
        where I: Iterator<Item = CmEvent<'a>>
    {
        ada.map(|hm_ev| {
               match hm_ev {
                   HmToken::StartTag{name, ..} | HmToken::EndTag{name} => name,
                   HmToken::Text(text) => text,
                   _ => panic!("Bad token {:?}", hm_ev),
               }
           })
           .collect()
    }

    #[test]
    fn text_grouping() {
        let md = "Multi\nLine\nText";
        let ada = Adapter::new(Parser::new(md), true);
        let res_vec = html_skel_map(ada);
        assert_eq!(&["p", md, "p"], &*res_vec);
    }

    #[test]
    fn code_grouping() {
        let code = "Multi\n\nline[2][3]\n\ncode\n\n";
        let md = String::from("```\n") + code + "```";
        let ada = Adapter::new(Parser::new(&*md), true);
        let res_vec = html_skel_map(ada);
        assert_eq!(&["pre", "code", code, "code", "pre"], &*res_vec);
    }
}
