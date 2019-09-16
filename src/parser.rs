use kuchiki::traits::*;
use kuchiki::{ElementData, NodeDataRef, NodeRef};
use serde_derive::Serialize;
use url::Url;

use super::{magnet_uri, size_parser};

#[derive(Debug, Serialize)]
pub struct Links {
    pub torrent: String,
    pub magnet: String,
}

#[derive(Debug, Serialize)]
pub struct Sizes {
    pub raw: String,
    pub parsed_from_magnet: Option<u64>,
    pub parsed_from_raw: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct NyaasiEntry {
    pub name: String,
    pub links: Links,
    pub sizes: Sizes,
    pub date: String,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
}

#[derive(Debug, Serialize)]
pub struct Page {
    pub url: String,
    pub number: u32,
}

#[derive(Debug, Serialize)]
pub struct Pagination {
    pub pages: Vec<Page>,
    pub current: Page,
}

#[derive(Debug, Serialize)]
pub struct Results {
    pub entries: Vec<NyaasiEntry>,
    pub pagination: Option<Pagination>,
}

impl Results {
    pub fn empty() -> Results {
        Results {
            entries: Vec::new(),
            pagination: None,
        }
    }
}

pub fn parse_html(html: &str, url: &str) -> Result<Results, String> {
    let current_url = Url::parse(url).map_err(|e| format!("Unable to parse url {}: {}", url, e))?;
    let dom = kuchiki::parse_html().one(html);

    let table = dom
        .select_first("tr.success > td.text-center > a > i.fa-magnet")
        .map_err(|()| "Unable to find first table row".to_owned())
        .and_then(|e| {
            e.as_node()
                .parent()
                .and_then(|e| e.parent())
                .and_then(|e| e.parent())
                .and_then(|e| e.parent())
                .and_then(|e| e.parent())
                .ok_or_else(|| "Unable to find table from first row".to_owned())
        })?;

    let mut entries = table
        .select("tr.success")
        .map_err(|()| "Unable to find table rows from table".to_owned())?
        .map(|row| {
            use std::str::FromStr;

            let magnet_uri = select_parent_href(
                row.as_node(),
                "td.text-center:nth-child(3) > a > i.fa-magnet",
                &current_url,
            )?;
            let magnet = magnet_uri::MagnetURI::from_str(&magnet_uri).ok();
            let raw_size = select_text(row.as_node(), "td.text-center:nth-child(4)")?;

            Ok(NyaasiEntry {
                name: select_text(row.as_node(), "td:nth-child(2) > a:not(.comments)")?,
                links: Links {
                    torrent: select_parent_href(
                        row.as_node(),
                        "td.text-center:nth-child(3) > a > i.fa-download",
                        &current_url,
                    )?,
                    magnet: magnet_uri,
                },
                sizes: Sizes {
                    raw: raw_size.clone(),
                    parsed_from_magnet: magnet.and_then(|m| m.length()),
                    parsed_from_raw: size_parser::parse(&raw_size).ok(),
                },
                date: select_text(row.as_node(), "td.text-center:nth-child(5)")?,
                seeders: select_u32(row.as_node(), "td.text-center:nth-child(6)")?,
                leechers: select_u32(row.as_node(), "td.text-center:nth-child(7)")?,
                downloads: select_u32(row.as_node(), "td.text-center:nth-child(8)")?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let pagination = match dom.select_first("ul.pagination > li.active > a") {
        Err(_) => None,
        Ok(current_page_node) => Some(make_page(&current_page_node, &current_url).and_then(
            |current| {
                Ok(Pagination {
                    pages: dom
                        .select("ul.pagination > li:not(.disabled) > a:not([rel])")
                        .map_err(|()| "Unable to find page list".to_owned())?
                        .map(|e| make_page(&e, &current_url))
                        .collect::<Result<Vec<_>, String>>()?,
                    current: current,
                })
            },
        )?),
    };

    //give newest last
    entries.reverse();

    Ok(Results {
        entries: entries,
        pagination: pagination,
    })
}

#[inline]
fn make_page(e: &NodeDataRef<ElementData>, current_url: &Url) -> Result<Page, String> {
    Ok(Page {
        url: href(e.as_node(), current_url)?,
        number: e
            .text_contents()
            .split_whitespace()
            .next()
            .ok_or_else(|| "Empty page element content".to_owned())
            .and_then(|n| {
                n.parse::<u32>()
                    .map_err(|e| format!("Unable to parse page number {} to u32: {}", n, e))
            })?,
    })
}

#[inline]
fn select_parent(node: &NodeRef, sel: &str) -> Result<NodeRef, String> {
    node.select_first(sel)
        .map_err(|()| format!("Unable to find element with {}", sel))?
        .as_node()
        .parent()
        .ok_or_else(|| format!("Unable to find parent of {}", sel))
}

#[inline]
fn select_text(node: &NodeRef, sel: &str) -> Result<String, String> {
    node.select_first(sel)
        .map_err(|()| format!("Unable to find element with {}", sel))
        .map(|e| e.text_contents())
}

#[inline]
fn select_u32(node: &NodeRef, sel: &str) -> Result<u32, String> {
    select_text(node, sel)?
        .parse::<u32>()
        .map_err(|e| format!("Unable to parse {} to u32: {}", sel, e))
}

#[inline]
fn select_parent_href(a: &NodeRef, sel: &str, current_url: &Url) -> Result<String, String> {
    href(&select_parent(a, sel)?, current_url)
}

#[inline]
fn href(a: &NodeRef, current_url: &Url) -> Result<String, String> {
    a.as_element()
        .ok_or_else(|| format!("Unable to convert {:?} to element", a))?
        .attributes
        .borrow()
        .get("href")
        .ok_or_else(|| "Unable to find href attribute".to_owned())
        .and_then(|url| {
            current_url
                .join(url)
                .map(|u| u.into_string())
                .map_err(|e| format!("Unable to join href url {} with page url: {}", url, e))
        })
}
