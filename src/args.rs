use clap::{App, Arg, ArgMatches};
use url::Url;

lazy_static! {
    static ref ARGS: ArgMatches<'static> = parse_args();
}

pub fn output_json() -> bool {
    ARGS.is_present("json")
}

pub fn get_url() -> Result<String, String> {
    Url::parse_with_params(
        "https://nyaa.si",
        &[
            ("f", try_u64("filter", "2")?),
            (
                "c",
                format!(
                    "{}_{}",
                    try_u64("category", "1")?,
                    try_u64("subcategory", "2")?
                ),
            ),
            ("p", try_u64("page", "1")?),
            ("q", ARGS.value_of("query").unwrap_or("").to_owned()),
        ],
    )
    .map(|u| u.into_string())
    .map_err(|e| e.to_string())
}

fn try_u64(name: &str, default: &str) -> Result<String, String> {
    match ARGS.value_of(name) {
        None => Ok(default.to_owned()),
        Some(v) => v
            .parse::<u64>()
            .map_err(|e| format!("Invalid value for {}: {}", name, e))
            .map(|_| v.to_owned()),
    }
}

fn parse_args() -> ArgMatches<'static> {
    App::new("nyaa.si")
        .version("0.1")
        .author("natanbc <natanbc@usp.br>")
        .about("Scrapes nyaa.si")
        .arg(Arg::with_name("json")
            .short("j")
            .long("json")
            .help("Output data as json instead"))
        .arg(Arg::with_name("filter")
            .short("f")
            .long("filter")
            .value_name("FILTER")
            .help("Sets the filter to apply - 0 is no filter, 1 is no remakes, 2 is trusted. Defaults to 2")
            .takes_value(true))
        .arg(Arg::with_name("category")
            .short("c")
            .long("category")
            .value_name("CATEGORY")
            .help("Sets the category wanted - 0 is all, 1 is anime, 2 is audio, 3 is literature, 4 is live action, 5 is pictures, 6 is software. Defaults to 1")
            .takes_value(true))
        .arg(Arg::with_name("subcategory")
            .short("s")
            .long("subcategory")
            .value_name("SUBCATEGORY")
            .help("Sets the subcategory wanted - 0 is all, non zero values depend on the type - for anime, 1 is AMV, 2 is english, 3 is non english, 4 is raw - for audio, 1 is lossless, 2 is lossy - for literature, 1 is english, 2 is non englis, 3 is raw - for live action, 1 is english, 2 is idol/promotional video, 3 is non english, 4 is raw - for pictures, 1 is graphics, 2 is photos - for software, 1 is applications, 2 is games. Defaults to 2")
            .takes_value(true))
        .arg(Arg::with_name("page")
             .short("p")
             .long("page")
             .value_name("PAGE")
             .help("Sets the page to load. Defaults to 1")
             .takes_value(true))
        .arg(Arg::with_name("query")
            .short("q")
            .long("query")
            .value_name("QUERY")
            .help("Sets the search query")
            .takes_value(true))
        .get_matches()
}
