mod api;
mod input;
mod schedule;
mod store;
mod translit;

use std::io::{BufRead, Write};
use std::time::{Duration, SystemTime};

fn prompt(stdin: &mut impl BufRead, msg: &str) -> Result<String, String> {
    print!("{msg}");
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let mut s = String::new();
    if stdin.read_line(&mut s).map_err(|e| e.to_string())? == 0 {
        return Err("unexpected end of input".into());
    }
    Ok(s.trim().to_string())
}

fn yes_no(stdin: &mut impl BufRead, msg: &str) -> Result<bool, String> {
    loop {
        match prompt(stdin, &format!("{msg} (y/n) "))?
            .to_lowercase()
            .as_str()
        {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {}
        }
    }
}

fn login(stdin: &mut impl BufRead) -> Result<api::Client, String> {
    if let Some(c) = store::load_credentials()
        && let Some(token) = &c.access_token
    {
        let client = api::Client::new(&c.domain, Some(token.clone()));
        if client.verify().is_ok() {
            return Ok(client);
        }
        println!("Stored login is no longer valid.");
    }
    let domain = api::normalize_domain(&prompt(stdin, "GoToSocial domain: ")?);
    let client = api::Client::new(&domain, None);
    let mut creds = client.register(&domain)?;
    println!(
        "Open this link, log in and copy the auth code:\n\n{}\n",
        client.authorize_url(&creds)
    );
    let code = prompt(stdin, "Auth code: ")?;
    let token = client.exchange_code(&creds, &code)?;
    let client = api::Client::new(&domain, Some(token.clone()));
    client.verify()?;
    creds.access_token = Some(token);
    store::save_credentials(&creds)?;
    println!("Logged in.");
    Ok(client)
}

fn read_addresses(stdin: &mut impl BufRead) -> Result<Vec<String>, String> {
    loop {
        println!("Paste addresses, end with a single \".\" line:");
        let lines = input::read_block(stdin).map_err(|e| e.to_string())?;
        let a = input::parse_addresses(&lines);
        if !a.is_empty() {
            return Ok(a);
        }
        println!("No addresses found.");
    }
}

fn run() -> Result<(), String> {
    let mut stdin = std::io::stdin().lock();
    let client = login(&mut stdin)?;
    let mut cache = store::load_cache();
    let x = cache
        .interval_secs
        .map_or(schedule::DEFAULT_X, Duration::from_secs);

    println!("Paste tab-separated rows, end with a single \".\" line:");
    let lines = input::read_block(&mut stdin).map_err(|e| e.to_string())?;
    let rows = input::parse_rows(&lines);
    if rows.is_empty() {
        return Err("no rows found".into());
    }
    let mut addresses = read_addresses(&mut stdin)?;
    let mut last = client.last_post_time()?;
    let mut posted = 0;

    for row in &rows {
        println!("\n=== {} ===", row.name);
        let address = loop {
            let mut found = None;
            for i in input::rank_addresses(&row.name, &addresses) {
                println!("\n{}\n", addresses[i]);
                if yes_no(&mut stdin, "Is this the address?")? {
                    found = Some(i);
                    break;
                }
            }
            match found {
                Some(i) => break addresses.remove(i),
                None => addresses = read_addresses(&mut stdin)?,
            }
        };
        let text = input::compose_post(row, &address);
        println!("\n{text}\n");
        let now = SystemTime::now();
        let at = schedule::next_time(now, last, x);
        let when = match at {
            Some(t) => {
                let dt = humantime::format_rfc3339_seconds(t);
                match last.and_then(|l| t.duration_since(l).ok()) {
                    Some(d) => format!("{dt} (+{})", humantime::format_duration(d)),
                    None => dt.to_string(),
                }
            }
            None => "now".to_string(),
        };
        if !yes_no(&mut stdin, &format!("Post this at {when}?"))? {
            addresses.push(address);
            continue;
        }
        client.post(&text, at)?;
        last = Some(at.unwrap_or(now));
        posted += 1;
        match at {
            Some(t) => println!("Scheduled for {}", humantime::format_rfc3339_seconds(t)),
            None => println!("Posted."),
        }
    }

    if posted > 0 {
        let avg = schedule::update_avg(cache.avg_batch, posted);
        cache.avg_batch = Some(avg);
        cache.interval_secs = Some(schedule::interval_for(avg).as_secs());
        store::save_cache(&cache)?;
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
