mod args;
pub mod parser;
pub mod magnet_uri;

fn main() {
    let url = match args::get_url() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let raw = reqwest::get(&url).and_then(|mut r| r.text()).expect("oof");

    let data = parser::parse(&raw).expect("Failed to parse");
    for row in data.entries.iter().rev() {
        println!("{}", row.name);
        println!("\tTorrent:    {}", row.links.torrent);
        println!("\tMagnet:     {}", row.links.magnet);
        println!("\tSize:       {} (parsed: {:?})", row.size, row.parsed_size);
        println!("\tDate added: {}", row.date);
        println!("\tSeeders:    {}", row.seeders);
        println!("\tLeechers:   {}", row.leechers);
        println!("\tDownloads:  {}", row.downloads);
    }
    print!("Pages: ");
    for page in data.pagination.pages {
        print!("{} ", page.number);
        if data.pagination.current.number == page.number {
            print!("(current) ");
        }
    }
    print!("\n");
}
