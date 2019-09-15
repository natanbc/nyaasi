#[macro_use]
extern crate lazy_static;

mod args;
pub mod magnet_uri;
pub mod parser;

fn main() {
    let url = match args::get_url() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let raw = reqwest::get(&url)
        .and_then(|mut r| r.text())
        .expect("Failed to fetch data from nyaa.si");

    let data = match parser::parse(&raw, &url) {
        None => {
            if args::output_json() {
                let serialized = serde_json::to_string(&parser::Results::empty())
                    .expect("Failed to serialize results");
                println!("{}", serialized);
            } else {
                eprintln!("Nothing found");
            }
            return;
        }
        Some(x) => x,
    };

    if args::output_json() {
        let serialized = serde_json::to_string(&data).expect("Failed to serialize results");
        println!("{}", serialized);
    } else {
        for row in data.entries.iter() {
            if args::should_print("name") {
                println!("{}", row.name);
            }
            if args::should_print("torrent") {
                println!("\tTorrent:    {}", row.links.torrent);
            }
            if args::should_print("magnet") {
                println!("\tMagnet:     {}", row.links.magnet);
            }
            if args::should_print("size") {
                print!("\tSize:       {}", row.size);
                if args::should_print("parsed_size") {
                    println!(" (parsed: {:?})", row.parsed_size);
                } else {
                    println!("");
                }
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
