use anyhow::Result;
use nom::{
    bytes::complete::{tag, take, take_while},
    character::complete::multispace0,
    IResult,
};
use std::collections::HashMap;

pub struct Parser<'a> {
    pub map: HashMap<&'a str, &'a str>,
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
                .map_err(|e| anyhow::anyhow!("Could not parse input data: {:?}", e))?;

            map.insert(key, value);

            head = input;
        }

        Ok(Self { map })
    }

    /// Gets a value from the container.  Same signature as HashMap::get
    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).copied()
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

fn parse_one_key_value(input: &str) -> IResult<&str, (&str, &str)> {
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

fn unquoted_value(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| !c.is_whitespace())(input)
}

fn quoted_value(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("\"")(input)?;

    // consume until we hit a quote
    let (input, so_far) = take_while(|c: char| c != '"')(input)?;

    let (input, _quote) = tag("\"")(input)?;

    Ok((input, so_far))
}

fn parse_value(input: &str) -> IResult<&str, &str> {
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
