//! Minimal markdown → HTML renderer for help content.
//!
//! Supports: `## / ### / ####` headings, blank-line paragraphs, `- ` / `* `
//! bullet lists, `**bold**`, `*italic*`, and `[text](url)` links. Everything
//! else is treated as plain text. All input is HTML-escaped before tags are
//! emitted, so it's safe to render into `inner_html`.

pub fn render(src: &str) -> String {
    let mut out = String::with_capacity(src.len() + 64);
    let mut lines = src.lines().peekable();
    let mut in_list = false;

    let close_list = |out: &mut String, in_list: &mut bool| {
        if *in_list {
            out.push_str("</ul>");
            *in_list = false;
        }
    };

    while let Some(line) = lines.next() {
        let trimmed = line.trim_end();

        // Blank line — paragraph break (and ends any open list).
        if trimmed.is_empty() {
            close_list(&mut out, &mut in_list);
            continue;
        }

        // Heading: up to four leading '#'. Fall through otherwise.
        if let Some((level, rest)) = heading(trimmed) {
            close_list(&mut out, &mut in_list);
            out.push_str(&format!("<h{level}>"));
            out.push_str(&render_inline(rest));
            out.push_str(&format!("</h{level}>"));
            continue;
        }

        // List item: "- " or "* " prefix.
        if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            if !in_list {
                out.push_str("<ul>");
                in_list = true;
            }
            out.push_str("<li>");
            out.push_str(&render_inline(rest));
            out.push_str("</li>");
            continue;
        }

        // Paragraph: gather consecutive non-blank, non-heading, non-list lines.
        close_list(&mut out, &mut in_list);
        let mut para = String::new();
        para.push_str(trimmed);
        while let Some(peek) = lines.peek() {
            let pt = peek.trim_end();
            if pt.is_empty()
                || heading(pt).is_some()
                || pt.starts_with("- ")
                || pt.starts_with("* ")
            {
                break;
            }
            para.push(' ');
            para.push_str(pt);
            lines.next();
        }
        out.push_str("<p>");
        out.push_str(&render_inline(&para));
        out.push_str("</p>");
    }
    close_list(&mut out, &mut in_list);
    out
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let level = trimmed.bytes().take_while(|&b| b == b'#').count();
    if (1..=4).contains(&level) {
        let rest = &trimmed[level..];
        if let Some(r) = rest.strip_prefix(' ') {
            return Some((level + 1, r)); // shift so '##' → h3 etc., leaving h1 for page
        }
    }
    None
}

/// Render inline markup: **bold**, *italic*, [text](url). Everything else is
/// HTML-escaped.
fn render_inline(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // **bold**
        if bytes[i] == b'*'
            && i + 1 < bytes.len()
            && bytes[i + 1] == b'*'
            && let Some(end) = find(src, i + 2, "**")
        {
            out.push_str("<strong>");
            out.push_str(&escape(&src[i + 2..end]));
            out.push_str("</strong>");
            i = end + 2;
            continue;
        }
        // *italic*
        if bytes[i] == b'*'
            && let Some(end) = find(src, i + 1, "*")
            && end > i + 1
        {
            out.push_str("<em>");
            out.push_str(&escape(&src[i + 1..end]));
            out.push_str("</em>");
            i = end + 1;
            continue;
        }
        // [text](url)
        if bytes[i] == b'['
            && let Some(close_bracket) = find(src, i + 1, "]")
            && close_bracket + 1 < bytes.len()
            && bytes[close_bracket + 1] == b'('
            && let Some(close_paren) = find(src, close_bracket + 2, ")")
        {
            let text = &src[i + 1..close_bracket];
            let url = &src[close_bracket + 2..close_paren];
            out.push_str("<a href=\"");
            out.push_str(&escape_attr(url));
            out.push_str("\">");
            out.push_str(&escape(text));
            out.push_str("</a>");
            i = close_paren + 1;
            continue;
        }
        // Plain char
        let ch_end = next_char_boundary(src, i);
        out.push_str(&escape(&src[i..ch_end]));
        i = ch_end;
    }
    out
}

fn find(s: &str, from: usize, needle: &str) -> Option<usize> {
    s.get(from..)
        .and_then(|slice| slice.find(needle).map(|p| from + p))
}

fn next_char_boundary(s: &str, from: usize) -> usize {
    let mut i = from + 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_heading_and_paragraph() {
        let out = render("## Title\n\nHello world.");
        assert_eq!(out, "<h3>Title</h3><p>Hello world.</p>");
    }

    #[test]
    fn renders_list() {
        let out = render("- a\n- b\n- c");
        assert_eq!(out, "<ul><li>a</li><li>b</li><li>c</li></ul>");
    }

    #[test]
    fn renders_bold_italic_and_link() {
        let out = render("This is **bold** and *em* and [link](https://x).");
        assert_eq!(
            out,
            "<p>This is <strong>bold</strong> and <em>em</em> and <a href=\"https://x\">link</a>.</p>"
        );
    }

    #[test]
    fn escapes_html_in_prose() {
        let out = render("<script>alert(1)</script>");
        assert_eq!(out, "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
    }

    #[test]
    fn escapes_url_quotes() {
        let out = render(r#"[x](http://a"b)"#);
        assert!(out.contains("href=\"http://a&quot;b\""));
    }

    #[test]
    fn multiline_paragraph_joins_with_space() {
        let out = render("line one\nline two\nline three");
        assert_eq!(out, "<p>line one line two line three</p>");
    }

    #[test]
    fn blank_line_starts_new_paragraph_and_closes_list() {
        let out = render("- a\n- b\n\npara");
        assert_eq!(out, "<ul><li>a</li><li>b</li></ul><p>para</p>");
    }

    #[test]
    fn unicode_content_preserved() {
        let out = render("héllo *wörld*");
        assert_eq!(out, "<p>héllo <em>wörld</em></p>");
    }
}
