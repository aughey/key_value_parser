use anyhow::Result;
use nom::{
    bytes::complete::{tag, take, take_while},
    character::complete::multispace0,
    IResult,
};
use std::collections::HashMap;

pub enum StringOrStr<'a> {
    String(String),
    Str(&'a str),
}
impl<'a> From<&'a str> for StringOrStr<'a> {
    fn from(value: &'a str) -> Self {
        StringOrStr::Str(value)
    }
}
impl From<String> for StringOrStr<'_> {
    fn from(value: String) -> Self {
        StringOrStr::String(value)
    }
}
impl AsRef<str> for StringOrStr<'_> {
    fn as_ref(&self) -> &str {
        match self {
            StringOrStr::String(s) => s.as_str(),
            StringOrStr::Str(s) => s,
        }
    }
}

pub struct Parser<'a> {
    map: HashMap<&'a str, StringOrStr<'a>>,
}
impl<'a> Parser<'a> {
    /// Construct a new parser.  
    /// If the parser cannot parse the input, an error will be returned.
    /// ```
    /// use key_value_parser::full_copy::Parser;
    /// const DATA: &str = "  key = value   ";
    /// let parser = Parser::new(DATA).unwrap();
    /// assert_eq!(parser.len(), 1);
    /// assert_eq!(parser.get("key").unwrap(), "value");
    /// ```
    pub fn new(input: &'a str) -> Result<Self> {
        // use nom to parse data
        let mut map = HashMap::new();

        let mut head = input.trim_start();
        while !head.is_empty() {
            let (input, (key, value)) = parse_one_key_value(head)
                .map_err(|e| anyhow::anyhow!("Could not parse input data: {} {:?}", input, e))?;

            map.insert(key, value);

            head = input;
        }

        Ok(Self { map })
    }

    /// Gets a value from the container.  Same signature as HashMap::get
    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|v| match v {
            StringOrStr::String(s) => s.as_str(),
            StringOrStr::Str(s) => s,
        })
    }

    /// Returns how many key value pairs are available
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if there are no key value pairs
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

fn parse_one_key_value(input: &str) -> IResult<&str, (&str, StringOrStr)> {
    // eat whitespace
    let (input, _) = multispace0(input)?;
    let (input, key) = take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)?;
    // eat whitespace
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("=")(input)?;
    // eat whitespace
    let (input, _) = multispace0(input)?;
    let (input, value) = parse_value(input)?;
    // eat whitespace
    let (input, _) = multispace0(input)?;

    Ok((input, (key, value)))
}

fn unquoted_value(input: &str) -> IResult<&str, StringOrStr> {
    let (input, value) = take_while(|c: char| !c.is_whitespace())(input)?;
    Ok((input, value.into()))
}

fn quoted_value(input: &str) -> IResult<&str, StringOrStr> {
    let (input, _) = tag("\"")(input)?;

    let mut accum: Option<String> = None;

    let mut head = input;
    loop {
        // consume until we hit a backslash or a quote
        let (input, so_far) = take_while(|c: char| c != '\\' && c != '"')(head)?;

        // let's see what we hit
        let (data, backslash_or_quote) = take(1usize)(input)?;

        match backslash_or_quote {
            "\"" => {
                // we hit a quote
                // so we're done
                let value = match accum {
                    Some(accum) => StringOrStr::String(accum + so_far),
                    None => StringOrStr::Str(so_far),
                };
                return Ok((data, value));
            }
            _ => {
                // we hit a backslash
                // so we need to see what's next
                let (data, next_char) = take(1usize)(data)?;
                // append that as a literal value
                let to_append = accum.get_or_insert_with(String::new);
                to_append.push_str(so_far);
                to_append.push_str(next_char);

                // move the head forward
                head = data;
            }
        }
    }
}

pub fn parse_value(input: &str) -> IResult<&str, StringOrStr> {
    let (_, peek_next_char) = take(1usize)(input)?;

    match peek_next_char {
        "\"" => quoted_value(input),
        _ => unquoted_value(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_happy_path() -> Result<()> {
        const DATA: &str = "  key = value   ";
        let parser = Parser::new(DATA)?;

        assert_eq!(parser.len(), 1);

        let value = parser.get("key").unwrap();
        assert_eq!(value, "value");

        Ok(())
    }

    #[test]
    fn test_happy_path_multiple_values() -> Result<()> {
        const DATA: &str = "  key = value  key2=value2 ";
        let parser = Parser::new(DATA)?;

        assert_eq!(parser.len(), 2);

        let value = parser.get("key").unwrap();
        assert_eq!(value, "value");

        let value = parser.get("key2").unwrap();
        assert_eq!(value, "value2");

        Ok(())
    }

    #[test]
    fn test_handles_quotes() -> Result<()> {
        const DATA: &str = "  key = \"value\"  key2=\"value2 with spaces\" ";
        let parser = Parser::new(DATA)?;

        assert_eq!(parser.len(), 2);

        let value = parser.get("key").unwrap();
        assert_eq!(value, "value");

        let value = parser.get("key2").unwrap();
        assert_eq!(value, "value2 with spaces");

        Ok(())
    }

    #[test]
    fn test_quoted_with_escape_characters() {
        const DATA: &str = "key=\"value with \\\"escaped\\\" quotes\"";

        let parser = Parser::new(DATA).unwrap();

        assert_eq!(parser.len(), 1);

        let value = parser.get("key").unwrap();
        assert_eq!(value, "value with \"escaped\" quotes");
    }

    #[test]
    fn test_bench_pattern() {
        const DATA: &str = "kkkkkkkkkk2=\"vvvvv\\\"ttttt2\" kkkkkkkkkk3=\"vvvvv\\\"ttttt3\" ";
        let parser = Parser::new(DATA).unwrap();
        assert_eq!(parser.len(), 2);
        let value = parser.get("kkkkkkkkkk2").unwrap();
        assert_eq!(value, "vvvvv\"ttttt2");
        let value = parser.get("kkkkkkkkkk3").unwrap();
        assert_eq!(value, "vvvvv\"ttttt3");
    }

    #[test]
    fn test_no_data() {
        const DATA: &str = "   ";

        let parser = Parser::new(DATA).unwrap();

        assert_eq!(parser.len(), 0);
    }

    #[test]
    fn test_bad_parsing() {
        const BAD_DATA: &[&str] = &[" foo ", "bar", ";foo=bar", "quoted=\"foo"];
        for data in BAD_DATA {
            let parser = Parser::new(data);
            assert!(parser.is_err(), "Should have failed to parse: {:?}", data);
        }
    }
}
