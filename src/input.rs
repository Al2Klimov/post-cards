use std::io::BufRead;

pub struct Row {
    pub month: u32,
    pub day: u32,
    pub name: String,
    pub ukraine: bool,
}

/// Reads lines until a line consisting solely of "." (or EOF).
pub fn read_block(input: &mut impl BufRead) -> std::io::Result<Vec<String>> {
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.trim() == "." {
            break;
        }
        lines.push(line.to_string());
    }
    Ok(lines)
}

fn parse_date(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.trim().split('/').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    let (m, d) = (parts[0].parse().ok()?, parts[1].parse().ok()?);
    if (1..=12).contains(&m) && (1..=31).contains(&d) {
        Some((m, d))
    } else {
        None
    }
}

pub fn parse_rows(lines: &[String]) -> Vec<Row> {
    lines.iter().filter_map(|l| parse_row(l)).collect()
}

fn parse_row(line: &str) -> Option<Row> {
    let cols: Vec<&str> = line.split('\t').collect();
    let (month, day) = cols.iter().find_map(|c| parse_date(c))?;
    let name = cols
        .iter()
        .map(|c| c.trim())
        .find(|c| !c.is_empty() && parse_date(c).is_none())?
        .to_string();
    let last = cols.last()?.trim();
    let mut chars = last.chars();
    let ukraine = matches!((chars.next(), chars.next()), (Some(c), None) if c.is_uppercase());
    Some(Row {
        month,
        day,
        name,
        ukraine,
    })
}

/// Splits pasted lines into addresses separated by blank lines.
pub fn parse_addresses(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for l in lines.iter().map(|l| l.trim()).chain(std::iter::once("")) {
        if l.is_empty() {
            if !cur.is_empty() {
                out.push(cur.join("\n"));
                cur.clear();
            }
        } else {
            cur.push(l);
        }
    }
    out
}

pub fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Indexes of addresses, best match for the name first.
pub fn rank_addresses(name: &str, addresses: &[String]) -> Vec<usize> {
    let score = |a: &String| levenshtein(name, a) as i64 - a.chars().count() as i64;
    let mut idx: Vec<usize> = (0..addresses.len()).collect();
    idx.sort_by_key(|&i| score(&addresses[i]));
    idx
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn compose_post(row: &Row, address: &str) -> String {
    let t = crate::translit::transliterate(&row.name);
    let parts: Vec<&str> = t.split_whitespace().collect();
    let name = match parts.as_slice() {
        [family, given, ..] => format!("{given} {family}"),
        _ => t.clone(),
    };
    let ua = if row.ukraine { " #Ukraine" } else { "" };
    format!(
        "Join me and wish {name}, #politicalPrisoner of #Russia, a #happyBirthday! ({} {}){ua}\n\nYou can e.g send a postcard to this address, like I did:\n\n{address}",
        MONTHS[row.month as usize - 1],
        row.day
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows() {
        let l = vec![
            "10/11/26\t09/27/26\tМарченко Александр Владимирович\tTRUE\tNut\tTRUE\t\tU".to_string(),
            "10/10/26\t09/26/26\tУстюгов Туран Павлович\tTRUE\tSwan\tTRUE\t\t".to_string(),
        ];
        let r = parse_rows(&l);
        assert_eq!(r.len(), 2);
        assert_eq!((r[0].month, r[0].day, r[0].ukraine), (10, 11, true));
        assert_eq!(r[0].name, "Марченко Александр Владимирович");
        assert!(!r[1].ukraine);
    }

    #[test]
    fn block_and_addresses() {
        let mut inp = "a\r\n\r\n b \n.\nignored\n".as_bytes();
        let b = read_block(&mut inp).unwrap();
        assert_eq!(parse_addresses(&b), vec!["a", "b"]);
    }

    #[test]
    fn lev() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn ranking() {
        let a = vec!["xxxx\nОльга".to_string(), "Иван Иванов, 1990".to_string()];
        assert_eq!(rank_addresses("Иван Иванов", &a)[0], 1);
    }

    #[test]
    fn post() {
        let r = Row {
            month: 10,
            day: 10,
            name: "Азатьян Георгий Самвелович".into(),
            ukraine: true,
        };
        let p = compose_post(&r, "ADDR");
        assert!(p.starts_with("Join me and wish Georgiy Azat'yan, #politicalPrisoner of #Russia, a #happyBirthday! (Oct 10) #Ukraine\n\n"));
        assert!(p.ends_with("like I did:\n\nADDR"));
    }
}
