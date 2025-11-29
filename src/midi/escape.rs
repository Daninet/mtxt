pub fn escape_string(s: &str) -> String {
    let mut output = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\0' => output.push_str("\\0"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\\' => output.push_str("\\\\"),
            c if c.is_control() => {
                output.push_str(&format!("\\x{:02x}", c as u32));
            }
            _ => output.push(c),
        }
    }
    output
}

pub fn unescape_string(s: &str) -> String {
    let mut output = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('0') => output.push('\0'),
                Some('n') => output.push('\n'),
                Some('r') => output.push('\r'),
                Some('t') => output.push('\t'),
                Some('\\') => output.push('\\'),
                Some('x') => {
                    // Expect 2 hex digits
                    let mut hex = String::new();
                    if let Some(h1) = chars.next() {
                        hex.push(h1);
                        if let Some(h2) = chars.next() {
                            hex.push(h2);
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                output.push(byte as char);
                            } else {
                                // Failed to parse hex, push raw chars
                                output.push_str("\\x");
                                output.push_str(&hex);
                            }
                        } else {
                            output.push_str("\\x");
                            output.push(h1);
                        }
                    } else {
                        output.push_str("\\x");
                    }
                }
                Some(other) => {
                    // Treat unknown escapes as the character itself.
                    output.push(other);
                }
                None => output.push('\\'), // Trailing backslash
            }
        } else {
            output.push(c);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(escape_string("Hello\nWorld"), "Hello\\nWorld");
        assert_eq!(escape_string("Null\0Byte"), "Null\\0Byte");
        assert_eq!(escape_string("Back\\Slash"), "Back\\\\Slash");
        assert_eq!(escape_string("\x01\x02"), "\\x01\\x02");
    }

    #[test]
    fn test_unescape() {
        assert_eq!(unescape_string("Hello\\nWorld"), "Hello\nWorld");
        assert_eq!(unescape_string("Null\\0Byte"), "Null\0Byte");
        assert_eq!(unescape_string("Back\\\\Slash"), "Back\\Slash");
        assert_eq!(unescape_string("\\x01\\x02"), "\x01\x02");
    }
}
