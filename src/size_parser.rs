lazy_static! {
    static ref SUFFIXES: Vec<&'static str> = vec!["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
}

/// Parses a size string such as "1.5 KiB" to a number in bytes.
/// 
/// Supported suffixes are B, KiB, MiB, GiB, TiB, PiB, EiB
///
/// # Examples
///
/// ```
/// let raw = "8 KiB";
/// let parsed = size_parser::parse(raw);
///
/// assert_eq!(parsed, Ok(8192));
/// ```
pub fn parse(size: &str) -> Result<u64, String> {
    let mut parts = size.split_whitespace();
    let raw_number = parts.next();
    let parsed_float = match raw_number.and_then(|s| s.parse::<f64>().ok()) {
        Some(v) => v,
        None => {
            return Err(match raw_number {
                Some(x) => format!("Failed to parse {} as f64", x),
                None => "Empty size string".to_owned(),
            })
        }
    };
    let raw_suffix = parts.next();
    if raw_suffix.is_none() {
        return Err("Unable to find size suffix".to_owned());
    }
    let suffix = raw_suffix.unwrap();
    let suffix_idx = match SUFFIXES.iter().position(|v| v == &suffix) {
        Some(v) => v,
        None => return Err(format!("Unable to find suffix {} in suffixes list", suffix)),
    };

    Ok((parsed_float * (1 << (10 * suffix_idx)) as f64) as u64)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_amount() {
        assert_eq!(Ok(8192), parse("8 KiB"));
    }

    #[test]
    fn test_floating_point_amount() {
        assert_eq!(Ok(1536), parse("1.5 KiB"));
    }

    #[test]
    fn test_empty() {
        assert_eq!(Err("Empty size string".to_owned()), parse(""));
    }

    #[test]
    fn test_invalid_float() {
        assert_eq!(Err("Failed to parse abc as f64".to_owned()), parse("abc KiB"));
    }

    #[test]
    fn test_no_suffix() {
        assert_eq!(Err("Unable to find size suffix".to_owned()), parse("1.2"));
    }

    #[test]
    fn test_invalid_suffix() {
        assert_eq!(Err("Unable to find suffix bits in suffixes list".to_owned()), parse("1.2 bits"));
    }
}
