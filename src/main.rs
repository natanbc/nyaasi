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
            println!("{}", row.name);
            println!("\tTorrent:    {}", row.links.torrent);
            println!("\tMagnet:     {}", row.links.magnet);
            println!("\tSize:       {} (parsed: {:?})", row.size, row.parsed_size);
            println!("\tDate added: {}", row.date);
            println!("\tSeeders:    {}", row.seeders);
            println!("\tLeechers:   {}", row.leechers);
            println!("\tDownloads:  {}", row.downloads);
        }
        if let Some(p) = data.pagination {
            print!("Pages: ");
            for page in p.pages {
                print!("{} ", page.number);
                if p.current.number == page.number {
                    print!("(current) ");
                }
            }
            print!("\n");
        }
    }
}
