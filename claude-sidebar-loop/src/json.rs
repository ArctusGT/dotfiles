//! Just enough JSON to read the sessions' files and `claude-sessions`'
//! lines, and to print a value the way jq does.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    /// as jq prints a number it has not changed: its literal, in decimal
    /// notation (`42.50` stays `42.50`, `1e2` is `1E+2`); see `jq_number`
    Number(String),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

static NULL: Value = Value::Null;

impl Value {
    /// field `key`, null when absent or not an object, as jq's `.key`
    pub fn get(&self, key: &str) -> &Value {
        match self {
            Value::Object(fields) => fields
                .iter()
                .find(|(k, _)| k == key)
                .map_or(&NULL, |(_, v)| v),
            _ => &NULL,
        }
    }

    /// sets field `key` of an object, in its place if present, else last,
    /// as jq's `.key = value`; nothing on any other value
    pub fn set(&mut self, key: &str, value: Value) {
        if let Value::Object(fields) = self {
            match fields.iter_mut().find(|(k, _)| k == key) {
                Some((_, v)) => *v = value,
                None => fields.push((key.to_string(), value)),
            }
        }
    }

    /// whether jq takes it as true: anything but null and false
    pub fn truthy(&self) -> bool {
        !matches!(self, Value::Null | Value::Bool(false))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => n.parse().ok(),
            _ => None,
        }
    }

    /// whether jq's `. > 0` holds: jq orders null, false, true, numbers,
    /// strings, arrays, objects
    pub fn above_zero(&self) -> bool {
        match self {
            Value::Null | Value::Bool(_) => false,
            Value::Number(_) => self.as_f64().is_some_and(|n| n > 0.0),
            _ => true,
        }
    }

    /// the value as jq interpolates it into a string, `"\(.)"`: a string as
    /// itself, anything else as its JSON
    pub fn text(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }
}

/// compact JSON, as jq's `tojson`
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => f.write_str(n),
            Value::String(s) => write_string(f, s),
            Value::Array(items) => {
                f.write_str("[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_str("]")
            }
            Value::Object(fields) => {
                f.write_str("{")?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write_string(f, k)?;
                    write!(f, ":{v}")?;
                }
                f.write_str("}")
            }
        }
    }
}

fn write_string(f: &mut fmt::Formatter, s: &str) -> fmt::Result {
    f.write_str("\"")?;
    for c in s.chars() {
        match c {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\t' => f.write_str("\\t")?,
            '\r' => f.write_str("\\r")?,
            '\u{8}' => f.write_str("\\b")?,
            '\u{c}' => f.write_str("\\f")?,
            c if c < ' ' || c == '\u{7f}' => write!(f, "\\u{:04x}", c as u32)?,
            c => write!(f, "{c}")?,
        }
    }
    f.write_str("\"")
}

/// every value in `text`, one after another, as `jq -s` reads a stream
pub fn parse_stream(text: &str) -> Result<Vec<Value>, String> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        at: 0,
    };
    let mut values = Vec::new();
    loop {
        parser.skip_space();
        if parser.at == parser.bytes.len() {
            return Ok(values);
        }
        values.push(parser.value()?);
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn skip_space(&mut self) {
        while self
            .bytes
            .get(self.at)
            .is_some_and(|b| b" \t\r\n".contains(b))
        {
            self.at += 1;
        }
    }

    fn fail<T>(&self, what: &str) -> Result<T, String> {
        Err(format!("{what} at byte {}", self.at))
    }

    fn next(&mut self) -> Option<u8> {
        let b = self.bytes.get(self.at).copied();
        self.at += 1;
        b
    }

    fn expect(&mut self, word: &str) -> Result<(), String> {
        if self.bytes[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            Ok(())
        } else {
            self.fail("unexpected input")
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        self.skip_space();
        match self.bytes.get(self.at) {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Value::String(self.string()?)),
            Some(b't') => self.expect("true").map(|_| Value::Bool(true)),
            Some(b'f') => self.expect("false").map(|_| Value::Bool(false)),
            Some(b'n') => self.expect("null").map(|_| Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => self.fail("unexpected input"),
        }
    }

    fn object(&mut self) -> Result<Value, String> {
        self.at += 1;
        let mut fields = Vec::new();
        self.skip_space();
        if self.bytes.get(self.at) == Some(&b'}') {
            self.at += 1;
            return Ok(Value::Object(fields));
        }
        loop {
            self.skip_space();
            if self.bytes.get(self.at) != Some(&b'"') {
                return self.fail("expected a key");
            }
            let key = self.string()?;
            self.skip_space();
            if self.next() != Some(b':') {
                return self.fail("expected :");
            }
            // a repeated key keeps its first place and its last value, as
            // jq reads it
            let value = self.value()?;
            match fields.iter_mut().find(|(k, _)| *k == key) {
                Some((_, v)) => *v = value,
                None => fields.push((key, value)),
            }
            self.skip_space();
            match self.next() {
                Some(b',') => {}
                Some(b'}') => return Ok(Value::Object(fields)),
                _ => return self.fail("expected , or }"),
            }
        }
    }

    fn array(&mut self) -> Result<Value, String> {
        self.at += 1;
        let mut items = Vec::new();
        self.skip_space();
        if self.bytes.get(self.at) == Some(&b']') {
            self.at += 1;
            return Ok(Value::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_space();
            match self.next() {
                Some(b',') => {}
                Some(b']') => return Ok(Value::Array(items)),
                _ => return self.fail("expected , or ]"),
            }
        }
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.at;
        while self
            .bytes
            .get(self.at)
            .is_some_and(|b| b"+-.eE0123456789".contains(b))
        {
            self.at += 1;
        }
        let literal = std::str::from_utf8(&self.bytes[start..self.at]).unwrap_or_default();
        match jq_number(literal) {
            Some(number) => Ok(Value::Number(number)),
            None => self.fail("bad number"),
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.at += 1;
        let mut out = Vec::new();
        loop {
            match self.next() {
                None => return self.fail("unterminated string"),
                Some(b'"') => break,
                Some(b'\\') => {
                    let c = match self.next() {
                        Some(b'"') => '"',
                        Some(b'\\') => '\\',
                        Some(b'/') => '/',
                        Some(b'b') => '\u{8}',
                        Some(b'f') => '\u{c}',
                        Some(b'n') => '\n',
                        Some(b'r') => '\r',
                        Some(b't') => '\t',
                        Some(b'u') => self.escaped_char()?,
                        _ => return self.fail("bad escape"),
                    };
                    out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
                }
                Some(b) => out.push(b),
            }
        }
        String::from_utf8(out).or_else(|_| self.fail("invalid UTF-8"))
    }

    /// the character a `\u` escape names, a surrogate pair taking two; a
    /// lone surrogate is U+FFFD, as jq reads it
    fn escaped_char(&mut self) -> Result<char, String> {
        let high = self.hex4()?;
        if (0xD800..0xDC00).contains(&high) && self.bytes[self.at..].starts_with(b"\\u") {
            let save = self.at;
            self.at += 2;
            let low = self.hex4()?;
            if (0xDC00..0xE000).contains(&low) {
                let c = 0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00);
                return Ok(char::from_u32(c).unwrap_or('\u{FFFD}'));
            }
            self.at = save;
        }
        Ok(char::from_u32(high).unwrap_or('\u{FFFD}'))
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let digits = self.bytes.get(self.at..self.at + 4);
        let n = digits
            .and_then(|d| std::str::from_utf8(d).ok())
            .and_then(|d| u32::from_str_radix(d, 16).ok());
        match n {
            Some(n) => {
                self.at += 4;
                Ok(n)
            }
            None => self.fail("bad \\u escape"),
        }
    }
}

/// number `literal` as jq 1.8 prints it unchanged: its decimal digits and
/// exponent as written, in the General Decimal Arithmetic's scientific
/// string (`to-sci-string`), so `0.1e1` is `1` and `12e3` is `1.2E+4`.
/// None if it is no number
fn jq_number(literal: &str) -> Option<String> {
    literal.parse::<f64>().ok()?;
    let (sign, unsigned) = match literal.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", literal),
    };
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(at) => (&unsigned[..at], unsigned[at + 1..].parse::<i64>().ok()?),
        None => (unsigned, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = format!("{whole}{fraction}");
    let digits = match digits.trim_start_matches('0') {
        "" => "0",
        significant => significant,
    };
    let exponent = exponent - fraction.len() as i64;
    let adjusted = exponent + digits.len() as i64 - 1;
    let text = if exponent <= 0 && adjusted >= -6 {
        let point = digits.len() as i64 + exponent;
        if exponent == 0 {
            digits.to_string()
        } else if point > 0 {
            format!(
                "{}.{}",
                &digits[..point as usize],
                &digits[point as usize..]
            )
        } else {
            format!("0.{}{digits}", "0".repeat(-point as usize))
        }
    } else {
        let (first, rest) = digits.split_at(1);
        let point = if rest.is_empty() { "" } else { "." };
        format!(
            "{first}{point}{rest}E{}{}",
            if adjusted < 0 { "-" } else { "+" },
            adjusted.abs()
        )
    };
    Some(format!("{sign}{text}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_tagged_line() {
        let values = parse_stream(
            "{\"header\":true,\"label\":\"local\"}\n{\"session_id\":\"a\\u00e9\\ud83d\\ude00\",\
             \"since\":1792310933,\"context\":{\"used_pct\":42.0},\"tmux_pane\":null}\n",
        )
        .unwrap();
        assert_eq!(values.len(), 2);
        assert!(values[0].get("header").truthy());
        assert!(!values[1].get("header").truthy());
        assert_eq!(values[1].get("session_id").as_str(), Some("aé😀"));
        assert_eq!(values[1].get("context").get("used_pct").text(), "42.0");
        assert_eq!(values[1].get("tmux_pane"), &Value::Null);
        assert_eq!(values[1].get("missing").get("deeper"), &Value::Null);
    }

    #[test]
    fn prints_values_as_jq_interpolates_them() {
        let values = parse_stream("[\"a\\\"b\\n\", 1.50, null, {\"k\": [true]}]").unwrap();
        let Value::Array(items) = &values[0] else {
            panic!()
        };
        let texts: Vec<String> = items.iter().map(Value::text).collect();
        assert_eq!(texts, ["a\"b\n", "1.50", "null", "{\"k\":[true]}"]);
        assert_eq!(items[0].to_string(), "\"a\\\"b\\n\"");
    }

    #[test]
    fn orders_against_zero_as_jq_does() {
        let values = parse_stream("null false true 0 2 -1 \"x\"").unwrap();
        let above: Vec<bool> = values.iter().map(Value::above_zero).collect();
        assert_eq!(above, [false, false, false, false, true, false, true]);
    }

    #[test]
    fn prints_numbers_as_jq_does() {
        let jq = [
            ("1e2", "1E+2"),
            ("1.0", "1.0"),
            ("42.50", "42.50"),
            ("-0", "-0"),
            ("-0.0", "-0.0"),
            ("0.1e1", "1"),
            ("0.0000001", "1E-7"),
            ("0.000001", "0.000001"),
            ("1.23e-5", "0.0000123"),
            ("12e3", "1.2E+4"),
            ("0e5", "0E+5"),
            ("-1.5E-3", "-0.0015"),
            ("1.5e300", "1.5E+300"),
            ("00012", "12"),
            ("0.00", "0.00"),
            (
                "123456789012345678901234567890",
                "123456789012345678901234567890",
            ),
        ];
        for (literal, printed) in jq {
            assert_eq!(
                parse_stream(literal).unwrap()[0].to_string(),
                printed,
                "{literal}"
            );
        }
    }

    #[test]
    fn a_repeated_key_keeps_its_place_and_last_value() {
        let mut value = parse_stream("{\"a\":1,\"b\":2,\"a\":3}").unwrap().remove(0);
        assert_eq!(value.to_string(), "{\"a\":3,\"b\":2}");
        value.set("b", Value::Null);
        value.set("c", Value::Bool(true));
        assert_eq!(value.to_string(), "{\"a\":3,\"b\":null,\"c\":true}");
    }

    #[test]
    fn rejects_broken_input() {
        assert!(parse_stream("{\"a\":}").is_err());
        assert!(parse_stream("[1,").is_err());
        assert!(parse_stream("\"open").is_err());
    }
}
