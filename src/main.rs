#[macro_use]
extern crate lazy_static;

mod args;

fn main() {
    let limit = match args::include_amount() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let url = match args::get_url() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let raw = match reqwest::get(&url).and_then(|mut r| r.text()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to fetch data: {}", e);
            return;
        }
    };

    let mut data = match nyaasi_scraper::parse_html(&raw, &url) {
        Err(e) => {
            eprintln!("{}", e);
            if args::output_json() {
                let serialized = serde_json::to_string(&nyaasi_scraper::Results::empty())
                    .expect("Failed to serialize results");
                println!("{}", serialized);
            }
            return;
        }
        Ok(x) => x,
    };

    data.entries = data
        .entries
        .into_iter()
        .rev()
        .take(limit)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    if args::output_json() {
        let serialized = serde_json::to_string(&data).expect("Failed to serialize results");
        println!("{}", serialized);
    } else {
        for row in data.entries.iter() {
            if args::should_print("name") {
                println!("{}", row.name);
            }
            if args::should_print("url") {
                println!("\tURL:        {}", row.url);
            }
            if args::should_print("kind") {
                println!("\tKind:       {:?}", row.kind);
            }
            if args::should_print("comments") {
                println!("\tComments:   {}", row.comments);
            }
            if args::should_print("torrent") {
                println!("\tTorrent:    {}", row.links.torrent);
            }
            if args::should_print("magnet") {
                println!("\tMagnet:     {}", row.links.magnet);
            }
            if args::should_print("size") {
                print!("\tSize:       {}", row.sizes.raw);
                let mut parsed: Vec<(&str, Option<u64>)> = Vec::new();
                if args::should_print("magnet_size") {
                    parsed.push(("magnet", row.sizes.parsed_from_magnet));
                }
                if args::should_print("parsed_size") {
                    parsed.push(("parsed", row.sizes.parsed_from_raw));
                }
                if parsed.len() > 0 {
                    print!(" (");
                    let mut comma = false;
                    for (name, v) in parsed.iter() {
                        if comma {
                            print!(", ");
                        }
                        comma = true;
                        print!("{}: {:?}", name, v);
                    }
                    print!(")");
                }
                println!("");
            }
            if args::should_print("date") {
                println!("\tDate added: {}", row.date);
            }
            if args::should_print("seeders") {
                println!("\tSeeders:    {}", row.seeders);
            }
            if args::should_print("leechers") {
                println!("\tLeechers:   {}", row.leechers);
            }
            if args::should_print("downloads") {
                println!("\tDownloads:  {}", row.downloads);
            }
        }
        if args::should_print("pages") {
            if let Some(p) = data.pagination {
                print!("Pages: ");
                for page in p.pages {
                    print!("{} ", page.number);
                    if p.current.number == page.number && args::should_print("current_page") {
                        print!("(current) ");
                    }
                }
                print!("\n");
            }
        }
    }
}
