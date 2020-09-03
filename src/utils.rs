use std::error::Error;
use std::path::PathBuf;
use std::fs::{create_dir_all, File};
use std::io::Write;

use scraper::html::Html;
use scraper::selector::Selector;
use url::Url;
use percent_encoding;

pub fn load(url: &str) -> Result<String, Box<dyn Error>> {
    Ok(reqwest::blocking::get(url)?.text()?)
}

pub fn urls(data: &str) -> Vec<String> {
    return Html::parse_document(data)
        .select(&Selector::parse("a").unwrap())
        .map(|v| v.value().attr("href"))
        .filter_map(|v| Some(v?.to_string()))
        .collect::<Vec<String>>();
}

pub fn add_prefix(url: &str, base: &str) -> Option<String> {
    let res = Url::parse(base)
        .and_then(|v| v.join(url));

    match res {
        Ok(v) => Some(v.into_string()),
        _ => None
    }
}

pub fn all_links(url: &str, data: &str) -> Vec<String> {
    urls(data)
        .into_iter()
        .filter_map(|link| add_prefix( &link, url))
        .collect()
}

pub fn decode(v: &str) -> String {
    percent_encoding::percent_decode_str(v).decode_utf8().unwrap().into_owned()
}

fn get_page_path(url: &str) -> (String, Vec<String>) {
    let purl = Url::parse(url).unwrap();
    let host_port = match purl.port() {
        None => purl.host_str().unwrap().to_string(),
        Some(v) => format!("{}:{}", purl.host_str().unwrap(), v)
    };

    let mut items = vec!["".to_string(), host_port];
    for part in purl.path_segments().unwrap() {
        items.push(part.to_string());
    }

    let mut fname = items.pop().unwrap();
    if fname.is_empty() {
        fname = "index.html".to_string();
    } else {
        fname = decode(&fname);
    }
    return (fname, items);
}

fn make_dirs(root: &PathBuf, items: &[String]) -> Result<PathBuf, Box<dyn Error>> {
    let mut cpath = root.clone();
    for itm in items {
        cpath = cpath.join(decode(&itm));
        if !cpath.exists() {
            create_dir_all(&cpath)?;
        }
        if !cpath.is_dir() {
            return Err(Box::new(simple_error::SimpleError::new("test".to_string())))
        }
    }
    Ok(cpath)
}

pub fn save_page(root: &PathBuf, url: &str, data: &str) -> Result<(), Box<dyn Error>> {
    let (fname, items) = get_page_path(url);
    let cpath = make_dirs(root, &items)?;
    File::create(cpath.join(fname))?.write_all(data.as_bytes())?;
    return Ok(())
}


