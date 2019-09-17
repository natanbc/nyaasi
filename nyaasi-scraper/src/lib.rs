#[macro_use]
extern crate lazy_static;

pub mod magnet_uri;
pub mod size_parser;

use kuchiki::traits::*;
use kuchiki::{ElementData, NodeDataRef, NodeRef};
use serde_derive::Serialize;
use url::Url;

use magnet_uri::MagnetURI;

/// Download links for an entry
#[derive(Debug, Serialize)]
pub struct Links {
    /// Link to download the torrent file
    pub torrent: String,
    /// Raw magnet uri
    pub magnet: String,
    /// Parsed magnet uri
    #[serde(skip_serializing)]
    pub parsed_magnet: Option<MagnetURI>,
}

/// Information about the size of an entry
#[derive(Debug, Serialize)]
pub struct Sizes {
    /// Raw, human readable string representing the size
    pub raw: String,
    /// Size parsed from the magnet uri. Currently (Sep 16, 2019), nyaasi
    /// doesn't return the size on any magnet uris, but this could change
    /// in the future
    pub parsed_from_magnet: Option<u64>,
    /// Size parsed from the raw string
    pub parsed_from_raw: Option<u64>,
}

/// Represents a download entry
#[derive(Debug, Serialize)]
pub struct NyaasiEntry {
    /// Name of the entry
    pub name: String,
    /// Number of comments on this entry
    pub comments: u32,
    /// Download links
    pub links: Links,
    /// Entry size
    pub sizes: Sizes,
    /// Date added
    pub date: String,
    /// Number of seeders
    pub seeders: u32,
    /// Number of leechers
    pub leechers: u32,
    /// Number of downloads completed
    pub downloads: u32,
}

/// Data about a page of the search query
#[derive(Debug, Serialize)]
pub struct Page {
    /// Url of the page. The html on this url can be provided to
    /// parse_html() to scrape it
    pub url: String,
    /// Number of the page
    pub number: u32,
}

/// Pagination data from a page
#[derive(Debug, Serialize)]
pub struct Pagination {
    /// List of pages around the current
    pub pages: Vec<Page>,
    /// Current page
    pub current: Page,
}

/// Data contained in a nyaa.si page.
#[derive(Debug, Serialize)]
pub struct Results {
    /// Entries in the page, in chronological order (aka newest last)
    pub entries: Vec<NyaasiEntry>,
    /// Pagination information extracted from the page.
    pub pagination: Option<Pagination>,
}

impl Results {
    /// Returns an empty result set.
    ///
    /// # Examples
    ///
    /// ```
    /// let r = Results::empty();
    ///
    /// assert_eq!(r.entries(), vec![]);
    /// assert_eq!(r.pagination, None);
    /// ```
    pub fn empty() -> Results {
        Results {
            entries: Vec::new(),
            pagination: None,
        }
    }
}

/// Parses HTML source and the page's url into a more usable format.
///
/// An error is returned if parsing fails.
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

            let raw_magnet = select_parent_href(
                row.as_node(),
                "td.text-center:nth-child(3) > a > i.fa-magnet",
                &current_url,
            )?;
            let magnet = MagnetURI::from_str(&raw_magnet).ok();
            let magnet_size = (&magnet).as_ref().and_then(|m| m.length());
            let raw_size = select_text(row.as_node(), "td.text-center:nth-child(4)")?;

            Ok(NyaasiEntry {
                name: select_text(row.as_node(), "td:nth-child(2) > a:not(.comments)")?,
                comments: select_text(row.as_node(), "td:nth-child(1) > a.comments > i")
                    .unwrap_or_else(|_| "0".to_owned())
                    .parse::<u32>()
                    .map_err(|e| format!("Unable to parse comment count as u32: {}", e))?,
                links: Links {
                    torrent: select_parent_href(
                        row.as_node(),
                        "td.text-center:nth-child(3) > a > i.fa-download",
                        &current_url,
                    )?,
                    magnet: raw_magnet,
                    parsed_magnet: magnet,
                },
                sizes: Sizes {
                    raw: raw_size.clone(),
                    parsed_from_magnet: magnet_size,
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
