//! This crate provides a simple key value parser.
//!
//! # Details
//!
//! This parser is designed to be simple and fast. It is not designed to be a full featured parser.
//!
//! If you're wanting to do config file parsing, please choose a more full featured parser and use an
//! established format like JSON, TOML, or YAML.

use anyhow::Result;
use nom::{
    bytes::complete::{tag, take, take_while},
    character::complete::multispace0,
    IResult,
};
use std::collections::HashMap;

pub struct Parser {
    pub map: HashMap<String, String>,
}
impl Parser {
    /// Construct a new parser.  
    /// If the parser cannot parse the input, an error will be returned.
    /// ```
    /// use key_value_parser::Parser;
    /// const DATA: &str = "  key = value   ";
    /// let parser = Parser::new(DATA).unwrap();
    /// assert_eq!(parser.len(), 1);
    /// assert_eq!(parser.get("key").unwrap(), "value");
    /// ```
    pub fn new(input: &str) -> Result<Self> {
        // use nom to parse data
        let mut map = HashMap::new();

        let mut head = input.trim_start();
        while !head.is_empty() {
            let (input, (key, value)) = parse_one_key_value(head)
                .map_err(|e| anyhow::anyhow!("Could not parse input data: {:?}", e))?;

            map.insert(key, value);

            head = input;
        }

        Ok(Self { map })
    }

    /// Gets a value from the container.  Same signature as HashMap::get
    pub fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
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

fn parse_one_key_value(input: &str) -> IResult<&str, (String, String)> {
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

    Ok((input, (key.to_string(), value)))
}

fn unquoted_value(input: &str) -> IResult<&str, String> {
    let (input, value) = take_while(|c: char| !c.is_whitespace())(input)?;
    Ok((input, value.to_string()))
}

fn quoted_value(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("\"")(input)?;

    // aaaaaaaaaaaa\"bbbbbbbbbbbbbbbb\"ccccccccccc\"dddddddddd
    let mut accum = String::new();

    let mut head = input;
    loop {
        // consume until we hit a backslash or a quote
        let (input, so_far) = take_while(|c: char| c != '\\' && c != '"')(head)?;

        accum.push_str(so_far);

        // let's see what we hit
        let (data, backslash_or_quote) = take(1usize)(input)?;

        match backslash_or_quote {
            "\"" => {
                // we hit a quote
                // so we're done
                head = data;
                break;
            }
            _ => {
                // we hit a backslash
                // so we need to see what's next
                let (data, next_char) = take(1usize)(data)?;
                // append that as a literal value
                accum.push_str(next_char);

                // move the head forward
                head = data;
            }
        }
    }

    Ok((head, accum))
}

fn parse_value(input: &str) -> IResult<&str, String> {
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
