use std::{borrow::Cow, cmp::Ordering, collections::VecDeque};

use regex::{Captures, Regex};
use wasm_bindgen::prelude::*;

#[derive(PartialEq)]
enum Status {
    Neutral,
    Table,
    List(usize),
    OrderedList(usize),
    Quote,
    Header,
}

fn html_escape(text: &str) -> Cow<'_, str> {
    let escape_re = Regex::new(r#"[<>&"'`]"#).unwrap();
    escape_re.replace_all(text, |caps: &Captures| match &caps[0] {
        "<" => "&lt;",
        ">" => "&gt;",
        "&" => "&amp;",
        "\"" => "&quot;",
        "'" => "&#x27;",
        "`" => "&#x60;",
        _ => "",
    })
}

#[wasm_bindgen]
pub fn parse(text: &str) -> String {
    let code_re = Regex::new(r"(?:^|(?:\n|\r\n|\r))\{code\}(?:\n|\r\n|\r)((?:.|\n|\r\n|\r)*?)(?:\n|\r\n|\r)\{/code\}(?:(?:\n|\r\n|\r)|$)").unwrap();
    let inline_code_re = Regex::new(r"\{code\}(.*?)\{/code\}").unwrap();
    let table_re = Regex::new(r"^\|(.*)\|h?$").unwrap();
    let list_re = Regex::new(r"^(-+)(.+)").unwrap();
    let ordered_list_re = Regex::new(r"^(\++)(.+)").unwrap();
    let header_re = Regex::new(r"^(\*{1,6})\s(.*)").unwrap();
    let italic_re = Regex::new(r"(&#x27;){3}(.*?)(&#x27;){3}").unwrap();
    let bold_re = Regex::new(r"(&#x27;){2}(.*?)(&#x27;){2}").unwrap();
    let strike_re = Regex::new(r"%%(.*?)%%").unwrap();
    let color_re = Regex::new(r"&amp;color\(\s*(.*?)\s*\)\s*\{\s*(.*?)\s*\}").unwrap();
    let quote_re = Regex::new(r"^&gt;(.*)").unwrap();
    let block_quote_re = Regex::new(r"(^|>)\{quote\}<br>(.*?)>\{/quote\}<br>").unwrap();
    let url_re = Regex::new(
        r"(?:\[\[([^\[\]]+?)(?:&gt;|:))?(https?://[\w!\?/\+\-_~=;\.,\*&@#\$%\(\)']+)(?:\]\])?",
    )
    .unwrap();

    let mut res = String::new();
    let mut previous_status = Status::Neutral;
    let mut code_stash: VecDeque<String> = VecDeque::from([]);
    const CODE_PLACEHOLDER: &str = "gPvErkJM67vL"; // This string has no meaning.

    let text = html_escape(text);

    // Stash code blocks.
    let text = code_re.replace_all(&text, |cap: &Captures| {
        let code = format!("<pre class=\"loom_code loom_code_cs\">{}</pre>", &cap[1]);
        code_stash.push_back(code);
        CODE_PLACEHOLDER
    });

    for line in text.lines() {
        let line = line.to_string();

        let line = inline_code_re.replace_all(
            &line,
            "<code class=\"prettyprint prettyprinted\" style><span class=\"typ\">$1</span></code>",
        );

        let current_status = if table_re.is_match(&line) {
            Status::Table
        } else if list_re.is_match(&line) {
            let caps = list_re.captures(&line).unwrap();
            let level = caps[1].len();
            Status::List(level)
        } else if ordered_list_re.is_match(&line) {
            let caps = ordered_list_re.captures(&line).unwrap();
            let level = caps[1].len();
            Status::OrderedList(level)
        } else if quote_re.is_match(&line) {
            Status::Quote
        } else if header_re.is_match(&line) {
            Status::Header
        } else {
            Status::Neutral
        };

        if previous_status != current_status {
            match previous_status {
                Status::Table => res.push_str("</tbody></table>"),
                Status::List(previous_level) => {
                    if let Status::List(level) = current_status {
                        if level < previous_level {
                            res.push_str(&"</li></ul>".repeat(previous_level - level))
                        }
                    } else {
                        res.push_str(&"</li></ul>".repeat(previous_level))
                    }
                }
                Status::OrderedList(previous_level) => {
                    if let Status::OrderedList(level) = current_status {
                        if level < previous_level {
                            res.push_str(&"</li></ol>".repeat(previous_level - level))
                        }
                    } else {
                        res.push_str(&"</li></ol>".repeat(previous_level))
                    }
                }
                Status::Quote => res.push_str("</blockquote>"),
                _ => (),
            }
        }

        match current_status {
            Status::Table => {
                if previous_status != Status::Table {
                    res.push_str("<table><tbody>");
                }

                res.push_str(
                    &(table_re
                        .captures(&line)
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str()
                        .split('|')
                        .fold(String::from("<tr>"), |acc, content| {
                            if line.ends_with('h') {
                                acc + "<th>" + content + "</th>"
                            } else if content.starts_with('~') {
                                acc + "<th>" + content.strip_prefix('~').unwrap() + "</th>"
                            } else {
                                acc + "<td>" + content + "</td>"
                            }
                        })
                        + "</tr>"),
                );
            }
            Status::List(level) => {
                if let Status::List(previous_level) = previous_status {
                    match level.cmp(&previous_level) {
                        Ordering::Greater => {
                            res.push_str(&"<ul><li>".repeat(level - previous_level))
                        }
                        Ordering::Equal => res.push_str("</li><li>"),
                        Ordering::Less => res.push_str("<li>"),
                    }
                } else {
                    res.push_str("<ul><li>")
                }

                res.push_str(&list_re.replace(&line, "$2"));
            }
            Status::OrderedList(level) => {
                if let Status::OrderedList(previous_level) = previous_status {
                    match level.cmp(&previous_level) {
                        Ordering::Greater => {
                            res.push_str(&"<ol><li>".repeat(level - previous_level))
                        }
                        Ordering::Equal => res.push_str("</li><li>"),
                        Ordering::Less => res.push_str("<li>"),
                    }
                } else {
                    res.push_str("<ol><li>")
                }

                res.push_str(&ordered_list_re.replace(&line, "$2"));
            }
            Status::Quote => {
                if previous_status != Status::Quote {
                    res.push_str("<blockquote>");
                }
                res.push_str(&quote_re.replace(&line, "$1<br>"));
            }
            Status::Header => {
                res.push_str(&header_re.replace(&line, |caps: &Captures| {
                    let level = &caps[1].len();
                    format!("<h{}>{}</h{}>", level, &caps[2], level)
                }));
            }
            Status::Neutral => {
                res.push_str(&line);
                res.push_str("<br>");
            }
        }

        previous_status = current_status;
    }

    match previous_status {
        Status::Table => res.push_str("</tbody></table>"),
        Status::List(previous_level) => res.push_str(&"</li></ul>".repeat(previous_level)),
        Status::OrderedList(previous_level) => res.push_str(&"</li></ol>".repeat(previous_level)),
        Status::Quote => res.push_str("</blockquote>"),
        _ => (),
    }

    let res = block_quote_re.replace_all(&res, |caps: &Captures| {
        format!(
            "{}<br><blockquote>{}></blockquote><br>",
            if &caps[1] == ">" { ">" } else { "" },
            &caps[2]
        )
    });

    // inline
    let res = italic_re.replace_all(&res, "<i>$2</i>");
    let res = bold_re.replace_all(&res, "<b>$2</b>");
    let res = strike_re.replace_all(&res, "<strike>$1</strike>");
    let res = color_re.replace_all(&res, |caps: &Captures| {
        let style = if caps[1].contains(',') {
            let s: Vec<&str> = caps[1].split(',').collect();
            format!("color: {};background-color: {};", s[0].trim(), s[1].trim())
        } else {
            format!("color: {};", &caps[1])
        };
        format!("<span style=\"{}\">{}</span>", style, &caps[2])
    });
    let res = url_re.replace_all(&res, |cap: &Captures| {
        format!(
            "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"loom-link-another\">{}</a>",
            &cap[2],
            cap.get(1).unwrap_or(cap.get(2).unwrap()).as_str()
        )});
    let res = res.replace("&amp;br;", "<br>");

    // Retrieve code blocks from the stash.
    let res = res
        .split(CODE_PLACEHOLDER)
        .fold(String::new(), |mut acc, str| {
            acc.push_str(str);
            acc.push_str(&code_stash.pop_front().unwrap_or(String::from("")));
            acc
        });

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_table() {
        let res = parse("|\n||aaa|bbb|h\n|ccc||~ddd|");
        assert_eq!(
            res,
            "|<br><table><tbody><tr><th></th><th>aaa</th><th>bbb</th></tr><tr><td>ccc</td><td></td><th>ddd</th></tr></tbody></table>"
        )
    }

    #[test]
    fn parse_orderd_list() {
        let res = parse("+ aaa");
        assert_eq!(res, "<ol><li> aaa</li></ol>")
    }

    #[test]
    fn parse_header() {
        let res = parse("*** これは見出しです。");
        assert_eq!(res, "<h3>これは見出しです。</h3>")
    }

    #[test]
    fn parse_italic() {
        let res = parse("これは'''イタリック'''です。");
        assert_eq!(res, "これは<i>イタリック</i>です。<br>");
    }

    #[test]
    fn parse_bold() {
        let res = parse("これは''ボールド''です。");
        assert_eq!(res, "これは<b>ボールド</b>です。<br>");
    }

    #[test]
    fn parse_quote() {
        let res = parse("\n{quote}\naaa\n{/quote}\n");
        assert_eq!(res, "<br><br><blockquote>aaa<br></blockquote><br>");
    }

    #[test]
    fn parse_url() {
        let res_1 = parse("[[Google>https://google.com]]");
        let res_2 = parse("[[Google:https://google.com]]");
        let res_3 = parse("https://google.com");
        assert_eq!(res_1, res_2);
        assert_eq!(res_1, "<a href=\"https://google.com\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"loom-link-another\">Google</a><br>");
        assert_eq!(res_3, "<a href=\"https://google.com\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"loom-link-another\">https://google.com</a><br>");
    }

    #[test]
    fn parse_inline_code() {
        let res = parse("aaa{code}bbb{/code}ccc");
        assert_eq!(res, "aaa<code class=\"prettyprint prettyprinted\" style><span class=\"typ\">bbb</span></code>ccc<br>");
    }

    #[test]
    fn parse_code() {
        let res = parse("aaa\n{code}\nhoge\nfuga\n{/code}\n");
        assert_eq!(
            res,
            "aaa<pre class=\"loom_code loom_code_cs\">hoge\nfuga</pre><br>"
        );
    }
}
