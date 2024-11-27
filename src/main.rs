use std::{
    path::PathBuf,
    fmt::Write,
    str::Split,
};
use regex::Regex;
use lazy_static::lazy_static;


#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum MdMode {
    #[default]
    None,
    Ul(char),
    Ol(u32),
    Code,
    Math,
    Quote,
    Table(u32, u32),
}


fn split_cols<'a>(line: &'a str) -> Option<Split<'a, char>> {
    if let Some(rest) = line.strip_prefix('|') {
        if let Some(rest) = rest.strip_suffix('|') {
            return Some(rest.split('|'))
        }
    }
    None
}


lazy_static! {
    static ref IMG: Regex = Regex::new(r"!\[\]\(([^\)]+)\)").unwrap();
    static ref LINK: Regex = Regex::new(r"\[([^\]]+)\]\(([^\)]+)\)").unwrap();
    static ref WIKI: Regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    static ref EMPH: Regex = Regex::new(r"\*\*\*([^\*]+)\*\*\*").unwrap();
    static ref EMPH2: Regex = Regex::new(r"_\*\*([^\*]+)\*\*_").unwrap();
    static ref EMPH3: Regex = Regex::new(r"__\*([^\*]+)\*__").unwrap();
    static ref EMPH4: Regex = Regex::new(r"\*\*_([^\*]+)_\*\*").unwrap();
    static ref EMPH5: Regex = Regex::new(r"\*__([^\*]+)__\*").unwrap();
    static ref EMPH6: Regex = Regex::new(r"___([^\*]+)___").unwrap();
    static ref BOLD: Regex = Regex::new(r"\*\*([^\*]+)\*\*").unwrap();
    static ref BOLD2: Regex = Regex::new(r"__([^\*]+)__").unwrap();
    static ref ITALIC: Regex = Regex::new(r"\*([^\*]+)\*").unwrap();
    static ref ITALIC2: Regex = Regex::new(r"_([^\*]+)_").unwrap();
    static ref CODE: Regex = Regex::new(r"`([^`]+)`").unwrap();
    static ref LINE: Regex = Regex::new(r"(\*\*\*+)|(---+)|(___+)").unwrap();
}


fn mode_none(content: &mut String, line: &str) -> Option<MdMode> {
    if line.trim().starts_with("$$") {
        writeln!(content, "{}", line.replace("<", "\\lt ")).ok()?;
        return Some(MdMode::Math);
    } else if let Some(rest) = line.strip_prefix("# ") {
        writeln!(content, "<h1>{}</h1>", parse_text(rest, false)?).ok()?;
    } else if let Some(rest) = line.strip_prefix("## ") {
        writeln!(content, "<h2>{}</h2>", parse_text(rest, false)?).ok()?;
    } else if let Some(rest) = line.strip_prefix("### ") {
        writeln!(content, "<h3>{}</h3>", parse_text(rest, false)?).ok()?;
    } else if let Some(rest) = line.strip_prefix("#### ") {
        writeln!(content, "<h4>{}</h4>", parse_text(rest, false)?).ok()?;
    } else if let Some(rest) = line.strip_prefix("##### ") {
        writeln!(content, "<h5>{}</h5>", parse_text(rest, false)?).ok()?;
    } else if let Some(rest) = line.strip_prefix("###### ") {
        writeln!(content, "<h6>{}</h6>", parse_text(rest, false)?).ok()?;
    } else if let Some(rest) = line.strip_prefix("- ") {
        writeln!(content, "<ul>\n<li>{}</li>", parse_text(rest, false)?).ok()?;
        return Some(MdMode::Ul('-'));
    } else if let Some(rest) = line.strip_prefix("* ") {
        writeln!(content, "<ul>\n<li>{}</li>", parse_text(rest, false)?).ok()?;
        return Some(MdMode::Ul('*'));
    } else if let Some(rest) = line.strip_prefix("1. ") {
        writeln!(content, "<ol>\n<li>{}</li>", parse_text(rest, false)?).ok()?;
        return Some(MdMode::Ol(2));
    } else if let Some(rest) = line.strip_prefix("> ") {
        writeln!(content, "<blockquote>{}", parse_text(rest, false)?).ok()?;
        return Some(MdMode::Quote);
    } else if let Some(rest) = line.strip_prefix("```") {
        writeln!(content, "<code><pre>{}", rest).ok()?;
        return Some(MdMode::Code);
    } else if let Some(cols) = split_cols(line) {
        writeln!(content, "<table>\n<tr>").ok()?;
        let mut count = 0;
        for c in cols {
            writeln!(content, "<th>{}</th>", c).ok()?;
            count += 1;
        }
        writeln!(content, "</tr>").ok()?;
        return Some(MdMode::Table(count, 1))
    } else if LINE.captures(line).is_some() {
        writeln!(content, "<hr />").ok()?;
    } else {
        writeln!(content, "{}", parse_text(line, true)?).ok()?;
    }
    Some(MdMode::None)
}

fn mode_ul(content: &mut String, line: &str, c: char) -> Option<MdMode> {
    if let Some(rest) = line.strip_prefix(&format!("{} ", c)) {
        writeln!(content, "<li>{}</li>", parse_text(rest, false)?).ok()?;
        Some(MdMode::Ul(c))
    } else {
        writeln!(content, "</ul>").ok()?;
        mode_none(content, line)
    }
}

fn mode_ol(content: &mut String, line: &str, n: u32) -> Option<MdMode> {
    if let Some(rest) = line.strip_prefix(&format!("{}. ", n)) {
        writeln!(content, "<li>{}</li>", parse_text(rest, false)?).ok()?;
        Some(MdMode::Ol(n + 1))
    } else {
        writeln!(content, "</ol>").ok()?;
        mode_none(content, line)
    }
}

fn mode_code(content: &mut String, line: &str) -> Option<MdMode> {
    if line.trim() == "```" {
        writeln!(content, "</pre></code>").ok()?;
        Some(MdMode::None)
    } else {
        writeln!(content, "{}", line).ok()?;
        Some(MdMode::Code)
    }
}

fn mode_math(content: &mut String, line: &str) -> Option<MdMode> {
    writeln!(content, "{}", line.replace("<", "\\lt ")).ok()?;
    if line.trim().ends_with("$$") {
        Some(MdMode::None)
    } else {
        Some(MdMode::Math)
    }
}

fn mode_quote(content: &mut String, line: &str) -> Option<MdMode> {
    if let Some(rest) = line.strip_prefix("> ") {
        writeln!(content, "{}", parse_text(rest, true)?).ok()?;
        Some(MdMode::Quote)
    } else if line == ">" {
        writeln!(content, "").ok()?;
        Some(MdMode::Quote)
    } else {
        writeln!(content, "</blockquote>").ok()?;
        mode_none(content, line)
    }
}

fn mode_table(content: &mut String, line: &str, n: u32, r: u32) -> Option<MdMode> {
    if let Some(cols) = split_cols(line) {
        if r == 1 {
            let mut count = 0;
            for c in cols {
                if !(c.trim().chars().all(|c| c == '-') && c.trim().len() >= 3) {
                    return None
                }
                count += 1;
            }
            if count != n { return None }
            Some(MdMode::Table(count, r + 1))
        } else {
            writeln!(content, "<tr>").ok()?;
            let mut count = 0;
            for c in cols {
                writeln!(content, "<td>{}</td>", parse_text(c, true)?).ok()?;
                count += 1;
            }
            if count != n { return None }
            writeln!(content, "</tr>").ok()?;
            Some(MdMode::Table(count, r + 1))
        }
    } else {
        writeln!(content, "</table>").ok()?;
        mode_none(content, line)
    }
}


fn parse_text(line: &str, p: bool) -> Option<String> {
    let content = line.replace("<", "\\lt ");
    let content = IMG.replace(&content, "<img src=\"$1\" />");
    let content = LINK.replace(&content, "<a href=\"$2\" >$1</a>");
    let content = WIKI.replace(&content, "<a href=\"$1\" >$1</a>");
    let content = EMPH.replace(&content, "<strong><em>$1</em></strong>");
    let content = EMPH2.replace(&content, "<strong><em>$1</em></strong>");
    let content = EMPH3.replace(&content, "<strong><em>$1</em></strong>");
    let content = EMPH4.replace(&content, "<strong><em>$1</em></strong>");
    let content = EMPH5.replace(&content, "<strong><em>$1</em></strong>");
    let content = EMPH6.replace(&content, "<strong><em>$1</em></strong>");
    let content = BOLD.replace(&content, "<strong>$1</strong>");
    let content = BOLD2.replace(&content, "<strong>$1</strong>");
    let content = ITALIC.replace(&content, "<em>$1</em>");
    let content = ITALIC2.replace(&content, "<em>$1</em>");
    let content = CODE.replace(&content, "<code>$1</code>");
    if p {
        Some(format!("<p>{}</p>", content))
    } else {
        Some(content.to_string())
    }
}


fn md_to_html(filename: &str, file: &str) -> Option<String> {
    let mut content = String::new();
    let mut mode = MdMode::None;

    for line in file.lines() {
        match mode {
            MdMode::None => mode = mode_none(&mut content, line)?,
            MdMode::Ul(c) => mode = mode_ul(&mut content, line, c)?,
            MdMode::Ol(n) => mode = mode_ol(&mut content, line, n)?,
            MdMode::Code => mode = mode_code(&mut content, line)?,
            MdMode::Math => mode = mode_math(&mut content, line)?,
            MdMode::Quote => mode = mode_quote(&mut content, line)?,
            MdMode::Table(n, r) => mode = mode_table(&mut content, line, n, r)?,
        }
    }

    Some(format!(

"<html><head>\n<title>{}</title>
<script type=\"text/x-mathjax-config\">
  MathJax.Hub.Config({{tex2jax: {{
      inlineMath: [['$','$']],
      displayMath: [['$$', '$$']]
  }}}});

</script>
<script type=\"text/javascript\" src=\"https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.1/MathJax.js?config=TeX-AMS-MML_HTMLorMML\">
</script>
<style>
* {{\n    box-sizing: border-box;\n}}
body {{\n    background-color: #222;\n    margin: 0 auto 0;\n    font-family: cmu serif;\n    font-size: 12pt;\n}}
#page {{\n    background-color: #FFF;\n    width: 8.3in;\n    height: auto;\n    padding: 0.6in;\n}}
table {{
    border-spacing: 0;
    border-collapse: separate;
    border-radius: 4px;
    border: 2px solid #BBB;
    font-size: 12pt;
}}
th {{\n    padding: auto;\n}}
td {{\n    padding: auto;\n}}
table th:not(:last-child), table td:not(:last-child) {{\n    border-right: 2px solid #BBB;\n}}
table>thead>tr:not(:last-child)>th,
table>thead>tr:not(:last-child)>td,
table>tbody>tr:not(:last-child)>th,
table>tbody>tr:not(:last-child)>td,
table>tfoot>tr:not(:last-child)>th,
table>tfoot>tr:not(:last-child)>td,
table>tr:not(:last-child)>td,
table>tr:not(:last-child)>th,
table>thead:not(:last-child),
table>tbody:not(:last-child),
table>tfoot:not(:last-child) {{\n    border-bottom: 2px solid #BBB;\n}}
img {{\n    max-width: 100%;\n}}
blockquote {{\n    background-color: #F8F8F8;\n    padding: 5px 0px 10px 5px;\n}}
code {{\n    background-color: #EEE;\n}}
pre {{\n    background-color: #EEE;\n    border-radius: 8px;\n    padding: 10px 0px 15px 15px;\n}}
</style>\n</head>\n<body>
<div id=\"page\">
{}
</div>
</body>\n</html>", filename, content

    ))
}


fn main() {
    let infile = std::env::args().nth(1).unwrap();
    let text = std::fs::read_to_string(&infile).unwrap();
    let temppath = PathBuf::from(&infile);
    let filename = temppath.file_name().unwrap().to_string_lossy();
    let mut outfile = std::path::PathBuf::from(infile);
    outfile.set_extension("md.html");
    std::fs::write(&outfile, md_to_html(&filename, &text).unwrap()).unwrap();

    std::process::Command::new("msedge")
        .arg(&outfile)
        .output()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));
    std::fs::remove_file(outfile).unwrap();
}

