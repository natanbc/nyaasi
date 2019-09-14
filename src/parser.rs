use kuchiki::traits::*;
use kuchiki::{ElementData, NodeRef, NodeDataRef};
use serde_derive::Serialize;

use super::magnet_uri;

#[derive(Debug, Serialize)]
pub struct Links {
    pub torrent: String,
    pub magnet: String
}

#[derive(Debug, Serialize)]
pub struct AnimeEntry {
    pub name: String,
    pub links: Links,
    pub size: String,
    pub parsed_size: Option<u64>,
    pub date: String,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32
}

#[derive(Debug, Serialize)]
pub struct Page {
    pub url: String,
    pub number: u32
}

#[derive(Debug, Serialize)]
pub struct Pagination {
    pub pages: Vec<Page>,
    pub current: Page
}

#[derive(Debug, Serialize)]
pub struct Results {
    pub entries: Vec<AnimeEntry>,
    pub pagination: Pagination
}

pub fn parse(html: &str) -> Option<Results> {
    let dom = kuchiki::parse_html().one(html);

    let table = dom.select_first("tr.success > td.text-center > a > i.fa-magnet")
        .ok().and_then(|e| e.as_node().parent()
                               .and_then(|e| e.parent())
                               .and_then(|e| e.parent())
                               .and_then(|e| e.parent())
                               .and_then(|e| e.parent())
        )?;

    let entries = table.select("tr.success").ok()?.map(|row| {
        use std::str::FromStr;

        let magnet_uri = select_parent_href(row.as_node(), "td.text-center:nth-child(3) > a > i.fa-magnet")?;
        let magnet = magnet_uri::MagnetURI::from_str(&magnet_uri).ok();

        Some(AnimeEntry {
            name: select_text(row.as_node(), "td:nth-child(2) > a:not(.comments)")?,
            links: Links {
                torrent: select_parent_href(row.as_node(), "td.text-center:nth-child(3) > a > i.fa-download")?,
                magnet: magnet_uri
            },
            size: select_text(row.as_node(), "td.text-center:nth-child(4)")?,
            parsed_size: magnet.and_then(|m| m.length()),
            date: select_text(row.as_node(), "td.text-center:nth-child(5)")?,
            seeders: select_u32(row.as_node(), "td.text-center:nth-child(6)")?,
            leechers: select_u32(row.as_node(), "td.text-center:nth-child(7)")?,
            downloads: select_u32(row.as_node(), "td.text-center:nth-child(8)")?
        })
    }).collect::<Option<Vec<_>>>()?;

    let pages = dom.select("ul.pagination > li:not(.disabled) > a:not([rel])").ok()?
        .map(|e| {
            make_page(&e)
        }).collect::<Option<Vec<_>>>()?;

    let current = make_page(
        &dom.select_first("ul.pagination > li.active > a").ok()?
    )?;


    Some(Results {
        entries: entries,
        pagination: Pagination {
            pages: pages,
            current: current
        }
    })
}

#[inline]
fn make_page(e: &NodeDataRef<ElementData>) -> Option<Page> {
    Some(Page {
        url: href(e.as_node())?,
        number: e.text_contents().split_whitespace().next()
            .and_then(|n| n.parse::<u32>().ok())?
    })
}

#[inline]
fn select_parent(node: &NodeRef, sel: &str) -> Option<NodeRef> {
    node.select_first(sel).ok()?.as_node().parent()
}

#[inline]
fn select_text(node: &NodeRef, sel: &str) -> Option<String> {
    node.select_first(sel).map(|e| e.text_contents()).ok()
}

#[inline]
fn select_u32(node: &NodeRef, sel: &str) -> Option<u32> {
    select_text(node, sel)?.parse::<u32>().ok()
}

#[inline]
fn select_parent_href(a: &NodeRef, sel: &str) -> Option<String> {
    href(&select_parent(a, sel)?)
}

#[inline]
fn href(a: &NodeRef) -> Option<String> {
    a.as_element()?.attributes.borrow().get("href").map(|e| e.to_owned()).map(|url| {
        if url.starts_with("/") {
            format!("https://nyaa.si{}", url)
        } else {
            url
        }
    })
}

