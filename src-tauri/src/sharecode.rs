//! Share code regex + `status` line parsing.
//!
//! Share codes come out of the console in the `CSGO-xxxxx-xxxxx-xxxxx-xxxxx-xxxxx`
//! format. We capture the raw string and forward it to cswatch.gg — decoding
//! happens server-side since we already have the implementation there.

use std::sync::LazyLock;

use regex::Regex;

pub static SHARECODE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // CSGO-AAAAA-BBBBB-CCCCC-DDDDD-EEEEE   (each block 5 chars, Steam's base32 alphabet)
    Regex::new(r"CSGO-[A-Za-z0-9]{5}-[A-Za-z0-9]{5}-[A-Za-z0-9]{5}-[A-Za-z0-9]{5}-[A-Za-z0-9]{5}")
        .expect("sharecode regex")
});

/// Matches a CS2 `status` output line of the form:
///   # <userid> <steamid64> "<name>" <connected> <ping> <loss> <state> <rate>
///
/// Returns (steam_id64, name). CS2 moved from STEAM_X:Y:Z format to raw 64-bit
/// IDs in the stringtable — support both just in case.
pub fn parse_status_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return None;
    }
    // Quick reject for the header line "# userid name uniqueid connected ..."
    if trimmed.contains("userid name") {
        return None;
    }

    // Find a quoted player name
    let q_start = trimmed.find('"')?;
    let q_end = trimmed[q_start + 1..].find('"')? + q_start + 1;
    let name = &trimmed[q_start + 1..q_end];

    // The steam id is the next whitespace-separated token after the close quote.
    let after = trimmed[q_end + 1..].trim_start();
    let token = after.split_whitespace().next()?;

    if let Some(id) = parse_steam_token(token) {
        Some((id, name.to_string()))
    } else {
        None
    }
}

fn parse_steam_token(token: &str) -> Option<String> {
    // Already a 17-digit steamid64
    if token.len() == 17 && token.chars().all(|c| c.is_ascii_digit()) {
        return Some(token.to_string());
    }
    // STEAM_1:0:12345 → 76561197960265728 + 2*Z + Y
    if let Some(rest) = token.strip_prefix("STEAM_") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 3 {
            let y: u64 = parts[1].parse().ok()?;
            let z: u64 = parts[2].parse().ok()?;
            let id: u64 = 76_561_197_960_265_728 + 2 * z + y;
            return Some(id.to_string());
        }
    }
    // [U:1:12345] → 76561197960265728 + 12345
    if let Some(rest) = token.strip_prefix("[U:1:").and_then(|s| s.strip_suffix(']')) {
        let z: u64 = rest.parse().ok()?;
        return Some((76_561_197_960_265_728 + z).to_string());
    }
    None
}
