/// Truncate a string with ellipsis in the middle
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    if max_len < 5 {
        return s[..max_len].to_string();
    }

    let half = (max_len - 3) / 2;
    let start = &s[..half];
    let end = &s[s.len() - half..];
    format!("{}...{}", start, end)
}

/// Format a nonce for display (truncated hex)
pub fn format_nonce(nonce: &str) -> String {
    truncate_string(nonce, 16)
}

/// Format an amount in millisatoshis for display
pub fn format_amount_msat(msat: u64) -> String {
    if msat >= 1_000_000_000 {
        let btc = msat as f64 / 100_000_000_000.0;
        format!("{:.8} BTC", btc)
    } else if msat >= 1_000_000 {
        let sats = msat / 1000;
        format!("{} sats", format_number(sats))
    } else if msat >= 1000 {
        let sats = msat / 1000;
        format!("{} sats", sats)
    } else {
        format!("{} msat", msat)
    }
}

/// Format a number with thousand separators
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "he...ld");
        assert_eq!(truncate_string("abcdefghij", 7), "ab...ij");
    }

    #[test]
    fn test_format_amount() {
        assert_eq!(format_amount_msat(500), "500 msat");
        assert_eq!(format_amount_msat(1000), "1 sats");
        assert_eq!(format_amount_msat(100000), "100 sats");
        assert_eq!(format_amount_msat(1000000), "1,000 sats");
        assert_eq!(format_amount_msat(100000000000), "1.00000000 BTC");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
    }
}
