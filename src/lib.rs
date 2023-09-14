use std::collections::HashMap;

use anyhow::Result;
use nom::{
    bytes::complete::{tag, take, take_while},
    character::complete::multispace0,
    IResult,
};

pub struct Parser {
    pub map: HashMap<String, String>,
}
impl Parser {
    pub fn new(input: &str) -> Result<Self> {
        // use nom to parse data
        let mut map = HashMap::new();

        let mut head = input;
        while !head.is_empty() {
            let (input, (key, value)) = parse_one_key_value(head)
                .map_err(|e| anyhow::anyhow!("Could not parse input data: {:?}", e))?;

            map.insert(key, value);

            head = input;
        }

        Ok(Self { map })
    }
    pub fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
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
}
