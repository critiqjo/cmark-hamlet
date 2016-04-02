use std::borrow::Cow;

use cmark::Tag as CmTag;
use cmark::Event as CmEvent;
use hamlet::Event as HmEvent;

pub struct Adapter<'a, I> {
    cm_iter: I,
    cm_looka: Option<CmEvent<'a>>, // for lookahead
    hm_queue: Vec<HmEvent<'a>>,
    group_text: bool,
}

impl<'a, I> Adapter<'a, I>
    where I: Iterator<Item = CmEvent<'a>>
{
    pub fn new<T>(iterable: T, group_text: bool) -> Adapter<'a, I>
        where T: IntoIterator<IntoIter = I, Item = CmEvent<'a>> + 'a
    {
        Adapter {
            cm_iter: iterable.into_iter(),
            cm_looka: None,
            hm_queue: Vec::with_capacity(2),
            group_text: group_text,
        }
    }

    fn cm_start_tag(&mut self, tag: CmTag<'a>) -> HmEvent<'a> {
        match tag {
            CmTag::Rule => {
                let _ = self.cm_iter.next(); // skip End(Rule)
                HmEvent::start_tag("hr", attr_set!()).closed()
            }
            CmTag::Code |
            CmTag::Strong |
            CmTag::Emphasis |
            CmTag::Paragraph |
            CmTag::BlockQuote |
            CmTag::Table(_) |
            CmTag::TableHead |
            CmTag::TableRow |
            CmTag::TableCell |
            CmTag::Item |
            CmTag::List(None) |
            CmTag::List(Some(1)) |
            CmTag::Header(_) => HmEvent::start_tag(tag_map(tag), attr_set!()),
            CmTag::List(Some(start)) => {
                HmEvent::start_tag("ol", attr_set!(start = format!("{}", start)))
            }
            CmTag::CodeBlock(lang) => {
                self.hm_queue.push(HmEvent::start_tag("code", attr_set!()));
                if lang.is_empty() {
                    HmEvent::start_tag("pre", attr_set!())
                } else {
                    HmEvent::start_tag("pre", attr_set!(dataLang = lang))
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
                let mut attrs = attr_set!(src = src);
                if !alt.is_empty() {
                    attrs.set_attr("alt", alt);
                }
                if !title.is_empty() {
                    attrs.set_attr("title", title);
                }
                HmEvent::start_tag("img", attrs).closed()
            }
            CmTag::Link(href, title) => {
                let mut attrs = attr_set!(href = href);
                if !title.is_empty() {
                    attrs.set_attr("title", title);
                }
                HmEvent::start_tag("a", attrs)
            }
            CmTag::FootnoteDefinition(_) => unimplemented!(),
        }
    }

    fn cm_end_tag(&mut self, tag: CmTag<'a>) -> HmEvent<'a> {
        match tag {
            CmTag::Rule => unreachable!(),
            CmTag::Code |
            CmTag::Strong |
            CmTag::Emphasis |
            CmTag::Paragraph |
            CmTag::BlockQuote |
            CmTag::Table(_) |
            CmTag::TableHead |
            CmTag::TableRow |
            CmTag::TableCell |
            CmTag::Item |
            CmTag::List(_) |
            CmTag::Header(_) => HmEvent::end_tag(tag_map(tag)),
            CmTag::CodeBlock(_) => {
                self.hm_queue.push(HmEvent::end_tag("pre"));
                HmEvent::end_tag("code")
            }
            CmTag::Image(_, _) => unreachable!(),
            CmTag::Link(_, _) => HmEvent::end_tag("a"),
            CmTag::FootnoteDefinition(_) => unimplemented!(),
        }
    }

    fn cm_text(&mut self, mut s: Cow<'a, str>) -> HmEvent<'a> {
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
        HmEvent::Text(s)
    }
}

fn tag_map<'a>(tag: CmTag<'a>) -> Cow<'a, str> {
    match tag {
        CmTag::Rule => "hr".into(),
        CmTag::Code => "code".into(),
        CmTag::Strong => "strong".into(),
        CmTag::Emphasis => "em".into(),
        CmTag::Paragraph => "p".into(),
        CmTag::BlockQuote => "blockquote".into(),
        CmTag::Table(_) => "table".into(),
        CmTag::TableHead | CmTag::TableRow => "tr".into(),
        CmTag::TableCell => "td".into(),
        CmTag::Item => "li".into(),
        CmTag::List(None) => "ul".into(),
        CmTag::List(Some(_)) => "ol".into(),
        CmTag::Header(level) => format!("h{}", level).into(),
        _ => unreachable!(),
    }
}

impl<'a, I> Iterator for Adapter<'a, I>
    where I: Iterator<Item = CmEvent<'a>>
{
    type Item = HmEvent<'a>;
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
                CmEvent::Html(html) | CmEvent::InlineHtml(html) => HmEvent::RawHtml(html),
                CmEvent::SoftBreak => self.cm_text("\n".into()),
                CmEvent::HardBreak => HmEvent::start_tag("br", attr_set!()).closed(),
                CmEvent::FootnoteReference(_) => unimplemented!(),
            };
            Some(hm_ev)
        }
    }
}

#[cfg(test)]
mod tests {
    use hamlet::Event as HmEvent;
    use cmark::Event as CmEvent;
    use cmark::Parser;
    use Adapter;
    use std::borrow::Cow;

    fn html_skel_map<'a, I>(ada: Adapter<'a, I>) -> Vec<Cow<'a, str>>
        where I: Iterator<Item = CmEvent<'a>>
    {
        ada.map(|hm_ev| {
               match hm_ev {
                   HmEvent::StartTag{name, ..} | HmEvent::EndTag{name} => name,
                   HmEvent::Text(text) => text,
                   _ => panic!("Bad event {:?}", hm_ev),
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
