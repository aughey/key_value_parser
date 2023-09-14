//! zero_parse will take a string like this:
//!
//! ```pre
//! one=1 two=2 three=three quoted="this is a quoted value" escaped="this is a value with \"escaped\" quotes"
//! ```
//!
use anyhow::Result;
use nom::{bytes::complete::{take_while, take, tag}, character::complete::multispace0, IResult, Finish};

use crate::full_almost_zero_copy::{StringOrStr, parse_value};

fn eat_value(input: &str) -> IResult<&str, ()> {
    let (_, peek_next_char) = take(1usize)(input)?;

    match peek_next_char {
        "\"" => eat_quoted_value(input),
        _ => eat_unquoted_value(input),
    }
}

fn eat_unquoted_value(input: &str) -> IResult<&str, ()> {
    let (input, _) = take_while(|c: char| !c.is_whitespace())(input)?;
    Ok((input, ()))
}

fn eat_quoted_value(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("\"")(input)?;

    let mut head = input;
    loop {
        // consume until we hit a backslash or a quote
        let (input, _) = take_while(|c: char| c != '\\' && c != '"')(head)?;

        // let's see what we hit
        let (data, backslash_or_quote) = take(1usize)(input)?;

        match backslash_or_quote {
            "\"" => {
                // we hit a quote
                // so we're done
                return Ok((data, ()));
            }
            _ => {
                // we hit a backslash
                // so we need to take what's next
                let (data, _) = take(1usize)(data)?;

                // move the head forward
                head = data;
            }
        }
    }
}


pub fn nom_parse<'a>(input: &'a str, search_key: &str) -> IResult<&'a str,StringOrStr<'a>> {
    let mut head = input.trim_start();

    while !head.is_empty() {
        // get next key
        let (input, _) = multispace0(head)?;
        let (input, key) =
            take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = tag("=")(input)?;
    
        // Found the key, extract the key, profit!
        if key == search_key {
            let (_,res) = parse_value(input)?;
            return Ok((input,res));
        } else {
            // eat the next value
            let (input,_) = eat_value(input)?;
            head = input;
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Fail,
    )))
}

pub fn parse<'a>(input: &'a str, search_key: &str) -> Result<StringOrStr<'a>> {
    let res = nom_parse(input,search_key).finish();
    match res {
        Ok((_,value)) => Ok(value),
        Err(e) => match e.code {
            nom::error::ErrorKind::Fail => Err(anyhow::anyhow!("Key not found")),
            _ => Err(anyhow::anyhow!("Error parsing input: {:?}",e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::full_almost_zero_copy;

    use super::*;

    #[test]
    fn test_happy_path() {
        const DATA: &str = "one=1 two=2 three=three quoted=\"this is a quoted value\" escaped=\"this is a value with \\\"escaped\\\" quotes\"";

        let parser = full_almost_zero_copy::Parser::new(DATA).unwrap();
        assert_eq!(parser.len(), 5);
        assert_eq!(parser.get("one").unwrap(), "1");
        assert_eq!(parser.get("two").unwrap(), "2");
        assert_eq!(parser.get("three").unwrap(), "three");
        assert_eq!(parser.get("quoted").unwrap(), "this is a quoted value");
        assert_eq!(
            parser.get("escaped").unwrap(),
            "this is a value with \"escaped\" quotes"
        );

        assert_eq!(parse(DATA, "one").unwrap().as_ref(), "1");
        assert_eq!(parse(DATA, "two").unwrap().as_ref(), "2");
        assert_eq!(parse(DATA, "three").unwrap().as_ref(), "three");
        assert_eq!(
            parse(DATA, "quoted").unwrap().as_ref(),
            "this is a quoted value"
        );
        assert_eq!(
            parse(DATA, "escaped").unwrap().as_ref(),
            "this is a value with \"escaped\" quotes"
        );

        assert!(parse(DATA, "four").is_err());
    }
}
