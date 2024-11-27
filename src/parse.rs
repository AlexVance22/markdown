use std::str::Lines;
use std::str::Split;


enum Item {
    H1(String),
    H2(String),
    H3(String),
    H4(String),
    H5(String),
    H6(String),
    Line,
    Table(Vec<Vec<Item>>),
    OrdList(Vec<Item>),
    UnordList(Vec<Item>),
    Text(String),
}


fn split_cols<'a>(line: &'a str) -> Option<Split<'a, char>> {
    if let Some(rest) = line.strip_prefix('|') {
        if let Some(rest) = rest.strip_prefix('|') {
            return Some(rest.split('|'))
        }
    }
    None
}



fn mode_none<'a>(content: &mut String, lines: Lines<'a>) -> Option<Vec<Item>> {
    let res = Vec::new();

    while let Some(line) = lines.next() {
        if let Some(rest) = line.strip_prefix("# ") {
            res.push(Item::H1(rest.to_string()));
        } else if let Some(rest) = line.strip_prefix("## ") {
            res.push(Item::H2(rest.to_string()));
        } else if let Some(rest) = line.strip_prefix("### ") {
            res.push(Item::H3(rest.to_string()));
        } else if let Some(rest) = line.strip_prefix("#### ") {
            res.push(Item::H4(rest.to_string()));
        } else if let Some(rest) = line.strip_prefix("##### ") {
            res.push(Item::H5(rest.to_string()));
        } else if let Some(rest) = line.strip_prefix("###### ") {
            res.push(Item::H6(rest.to_string()));
        } else if let Some(rest) = line.strip_prefix("- ") {
            res.push(Item::UnordList(mode_ul(content, lines, line)?));
        } else if let Some(rest) = line.strip_prefix("* ") {
            res.push(Item::UnordList(mode_ul(content, lines, line)?));
        } else if let Some(rest) = line.strip_prefix("1. ") {
            res.push(Item::OrdList(mode_ol(content, lines, line)?));
        } else if let Some(cols) = split_cols(line) {
            writeln!(content, "<table>\n<tr>").ok()?;
            let mut count = 0;
            for c in cols {
                writeln!(content, "<th>{}</th>", c).ok()?;
                count += 1;
            }
            writeln!(content, "\n</tr>").ok()?;
            return Some(MdMode::Table(count))
        } else if (line.chars().all(|c| c == '-') || line.chars().all(|c| c == '*')) && line.len() >= 3 {
            writeln!(content, "<hr />").ok()?;
        } else {
            writeln!(content, "{}", line).ok()?;
        }
    }

    Some(res)
}

fn mode_ul<'a>(content: &mut String, lines: Lines<'a>, line: &str) -> Option<Vec<Item>> {
    let res = Vec::new();
    let c = line.chars().next().unwrap();
    if let Some(rest) = line.strip_prefix(&format!("{} ", c)) {
    }

    while let Some(line) = lines.next() {
        if let Some(rest) = line.strip_prefix(&format!("{} ", c)) {
            res.push(Item::Text(rest.to_string()));
        } else {
            mode_none(content, line);
        }
    }

    Some(res)
}

fn mode_ul<'a>(content: &mut String, lines: Lines<'a>, line: &str) -> Option<Vec<Item>> {
    let res = Vec::new();

    if let Some(rest) = line.strip_prefix(&format!("{}. ", n)) {
        writeln!(content, "<li>{}</li>", rest).ok()?;
        Some(MdMode::Ol(n + 1))
    } else {
        writeln!(content, "</ol>").ok()?;
        mode_none(content, line)
    }

    Some(res)
}

fn mode_table(content: &mut String, line: &str, n: u32) -> Option<MdMode> {
    if let Some(cols) = split_cols(line) {
        writeln!(content, "<tr>").ok()?;
        let mut count = 0;
        for c in cols {
            writeln!(content, "<td>{}</td>", c).ok()?;
            count += 1;
        }
        if count != n { return None }
        writeln!(content, "\n</tr>").ok()?;
        return Some(MdMode::Table(count))
    } else {
        writeln!(content, "</table>").ok()?;
        mode_none(content, line)
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
            MdMode::Table(n) => mode = mode_table(&mut content, line, n)?,
        }
    }

    Some(format!(

"<html><head>\n<title>{}</title>\n<style>
* {{\n    box-sizing: border-box;\n}}
body {{\n    background-color: #222;\n    margin: 0 auto 0;\n}}
#page {{\n    background-color: #FFF;\n    font-size: 15pt;\n    width: 8.3in;\n    height: auto;\n    padding: 1in;\n}}
</style>\n</head>\n<body>
<div id=\"page\">
{}
</div>
</body>\n</html>", filename, content

    ))
}
