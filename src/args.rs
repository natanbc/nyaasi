use clap::{App, AppSettings, Arg, ArgMatches};
use url::Url;

lazy_static! {
    static ref ARGS: ArgMatches<'static> = parse_args();
    static ref FILTERS: Vec<&'static str> = vec!["No filter", "No remakes", "Trusted only",];
    static ref NYAASI_CATEGORIES: Vec<Category> = vec![
        Category::from("All categories", vec![]),
        Category::from(
            "Anime",
            vec![
                "Anime Music Video",
                "English-translated",
                "Non-English-translated",
                "Raw"
            ]
        ),
        Category::from("Audio", vec!["Lossless", "Lossy"]),
        Category::from(
            "Literature",
            vec!["English-translated", "Non-English-translated", "Raw"]
        ),
        Category::from(
            "Live Action",
            vec![
                "English-translated",
                "Idol/Promotional Video",
                "Non-English-translated",
                "Raw"
            ]
        ),
        Category::from("Pictures", vec!["Graphics", "Photos"]),
        Category::from("Software", vec!["Applications", "Games"]),
    ];
    static ref SUKEBEI_CATEGORIES: Vec<Category> = vec![
        Category::from("All categories", vec![]),
        Category::from(
            "Art",
            vec!["Anime", "Doujinshi", "Games", "Manga", "Pictures"]
        ),
        Category::from("Real Life", vec!["Photobooks and Pictures", "Videos"]),
    ];
}

enum Source {
    NYAASI,
    SUKEBEI,
}

impl Source {
    fn categories(&self) -> &'static Vec<Category> {
        match self {
            Source::NYAASI => &NYAASI_CATEGORIES,
            Source::SUKEBEI => &SUKEBEI_CATEGORIES,
        }
    }

    fn base_url(&self) -> &'static str {
        match self {
            Source::NYAASI => "https://nyaa.si",
            Source::SUKEBEI => "https://sukebei.nyaa.si",
        }
    }

    fn parse_categories(&self) -> Result<(usize, usize), String> {
        let categories = self.categories();
        let category_idx = try_parse("category", 0usize)?;
        let subcategory_idx = try_parse("subcategory", 0usize)?;

        if category_idx >= categories.len() {
            return Err(format!(
                "Category out of bounds: {} available, got {}",
                categories.len(),
                category_idx
            ));
        }

        let subcategories = &categories[category_idx].subcategories;

        if subcategories.len() == 0 {
            if subcategory_idx != 0 {
                return Err(format!(
                    "Subcategory must be 0 for categories without subcategories, got {}",
                    subcategory_idx
                ));
            }
        } else if subcategory_idx >= subcategories.len() {
            return Err(format!(
                "Subcategory out of bounds: {} available, for {}",
                subcategories.len(),
                subcategory_idx
            ));
        }

        Ok((category_idx, subcategory_idx))
    }
}

struct Category {
    name: &'static str,
    subcategories: Vec<&'static str>,
}

impl Category {
    fn from(name: &'static str, subcategories: Vec<&'static str>) -> Category {
        Category {
            name: name,
            subcategories: subcategories,
        }
    }

    fn names(list: &Vec<Category>) -> String {
        list.iter()
            .enumerate()
            .map(|(index, category)| format!("{} - {}", index, category.name))
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn names_and_subcategories(list: &Vec<Category>) -> String {
        list.iter()
            .enumerate()
            .map(|(index, category)| {
                if category.subcategories.len() == 0 {
                    format!("{} - {}", index, category.name)
                } else {
                    format!(
                        "{} - {}\n{}",
                        index,
                        category.name,
                        category
                            .subcategories
                            .iter()
                            .enumerate()
                            .map(|(i, s)| format!("   {} - {}", i + 1, s))
                            .collect::<Vec<String>>()
                            .join("\n")
                    )
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

pub fn include_amount() -> Result<usize, String> {
    try_parse("number", 10000000usize)
}

pub fn output_json() -> bool {
    ARGS.is_present("json")
}

pub fn should_print(what: &str) -> bool {
    match ARGS.values_of("include") {
        None => true,
        Some(mut v) => v.any(|v| v == what),
    }
}

pub fn get_url() -> Result<String, String> {
    let source = match ARGS.value_of("source") {
        None => Source::NYAASI,
        Some("nyaasi") => Source::NYAASI,
        Some("sukebei") => Source::SUKEBEI,
        Some(src) => return Err(format!("Invalid source {}", src)),
    };
    let filter = try_parse("filter", 2usize)?;
    let (category, subcategory) = source.parse_categories()?;

    if filter >= FILTERS.len() {
        return Err(format!(
            "Filter out of bounds: {} available, got {}",
            FILTERS.len(),
            filter
        ));
    }

    Url::parse_with_params(
        source.base_url(),
        &[
            ("f", filter.to_string()),
            ("c", format!("{}_{}", category, subcategory)),
            ("p", try_parse("page", 1u64)?.to_string()),
            ("q", ARGS.value_of("query").unwrap_or("").to_owned()),
        ],
    )
    .map(|u| u.into_string())
    .map_err(|e| e.to_string())
}

fn try_parse<T>(name: &str, default: T) -> Result<T, String>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match ARGS.value_of(name) {
        None => Ok(default),
        Some(v) => v
            .parse::<T>()
            .map_err(|e| format!("Invalid value for {}: {}", name, e)),
    }
}

fn parse_args() -> ArgMatches<'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("natanbc <natanbc@usp.br>")
        .about("Scrapes nyaa.si")
        .global_setting(AppSettings::ColoredHelp)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .arg(Arg::with_name("source")
            .short("S")
            .long("source")
            .value_name("SOURCE")
            .help("Selects the source")
            .takes_value(true)
            .possible_values(&vec!["nyaasi", "sukebei"])
            .default_value("nyaasi"))
        .arg(Arg::with_name("filter")
            .short("f")
            .long("filter")
            .value_name("FILTER")
            .help("Sets the filter to apply - 0 is no filter, 1 is no remakes, 2 is trusted")
            .takes_value(true)
            .default_value("2"))
        .arg(Arg::with_name("category")
            .short("c")
            .long("category")
            .value_name("CATEGORY")
            .help("Sets the category wanted")
            .long_help(&format!(
                    "Sets the category wanted\nNyaa.si categories:\n{}\n\nSukebei categories:\n{}",
                    Category::names(&NYAASI_CATEGORIES),
                    Category::names(&SUKEBEI_CATEGORIES)
            ))
            .takes_value(true))
        .arg(Arg::with_name("subcategory")
            .short("s")
            .long("subcategory")
            .value_name("SUBCATEGORY")
            .help("Sets the subcategory wanted")
            .long_help(&format!(
                    "Sets the subcategory wanted\nNyaa.si subcategories:\n{}\n\nSukebei categories:\n{}",
                    Category::names_and_subcategories(&NYAASI_CATEGORIES),
                    Category::names_and_subcategories(&SUKEBEI_CATEGORIES)
            ))
            .takes_value(true))
        .arg(Arg::with_name("query")
            .short("q")
            .long("query")
            .value_name("QUERY")
            .help("Sets the search query")
            .takes_value(true))
        .arg(Arg::with_name("page")
             .short("p")
             .long("page")
             .value_name("PAGE")
             .help("Sets the page to load")
             .takes_value(true)
             .default_value("1"))
        .arg(Arg::with_name("include")
            .short("i")
            .long("include")
            .value_name("FIELD")
            .help("Includes a field when printing to stdout. Ignored if --json is present.\nValid values are name, torrent, magnet, size, magnet_size, parsed_size, date, seeders, leechers, downloads, pages, current_page.\nIgnores parsed_size and magnet_size if size is not present.\nIgnores current_page if pages is not set")
            .takes_value(true)
            .multiple(true))
        .arg(Arg::with_name("number")
            .short("n")
            .long("number")
            .value_name("AMOUNT")
            .help("Number of elements to include. Only the <AMOUNT> most recent ones will be included")
            .takes_value(true))
        .arg(Arg::with_name("json")
            .short("j")
            .long("json")
            .help("Output data as json instead"))
        .get_matches()
}
