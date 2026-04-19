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

/// Matches a `status` player row across both CS:GO-era and live CS2 formats.
///
/// CS:GO / offline / GOTV / demo (steam id exposed):
///   # 3 <steamid64> "playerName" 01:23 45 0 active 64
///
/// CS2 live (no steam id — anti-cheat policy):
///   [Client] 65285    04:46   15    0     active 786432 'GGDelta | Mr Cheng'
///
/// Returns (steam_id_or_synthetic, name). For live CS2 rows without a steam
/// id, we synthesize `cs2name:<name>` so the UI can still show a roster even
/// though lookup against cswatch.gg is impossible until a share code arrives.
pub fn parse_status_line(line: &str) -> Option<(String, String)> {
    // Strip CS2 log prefix tags like "[Client] " / "[EngineServiceManager] ".
    let mut trimmed = line.trim();
    while trimmed.starts_with('[') {
        if let Some(end) = trimmed.find(']') {
            trimmed = trimmed[end + 1..].trim_start();
        } else {
            break;
        }
    }

    // Reject obvious header / control lines.
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("userid name")
        || lower.starts_with("name time")
        || lower.starts_with("id     time")
        || lower.starts_with("# userid")
        || lower.contains("-----")
        || lower.contains("spawngroup")
        || lower.contains("status -----")
        || lower.contains("challenging")
        || lower.contains("[nochan]")
        || lower.contains("'demorecorder'")
    {
        return None;
    }

    // Strategy 1: classic CS:GO / demo row with double-quoted name + steam id.
    if trimmed.starts_with('#') {
        if let Some(q_start) = trimmed.find('"') {
            if let Some(rel_end) = trimmed[q_start + 1..].find('"') {
                let q_end = rel_end + q_start + 1;
                let name = &trimmed[q_start + 1..q_end];
                let after = trimmed[q_end + 1..].trim_start();
                if let Some(token) = after.split_whitespace().next() {
                    if let Some(id) = parse_steam_token(token) {
                        return Some((id, name.to_string()));
                    }
                }
                // Quoted name but no resolvable steam id — fall through.
                return Some((format!("cs2name:{name}"), name.to_string()));
            }
        }
    }

    // Strategy 2: CS2 live format with single-quoted name and no steam id
    // column at all. Only accept rows that look like the active-player table.
    if let Some(q_start) = trimmed.find('\'') {
        if let Some(rel_end) = trimmed[q_start + 1..].rfind('\'') {
            let q_end = rel_end + q_start + 1;
            let name = trimmed[q_start + 1..q_end].trim();
            let head = &trimmed[..q_start];
            let tail = &trimmed[q_end + 1..];

            // Must be an "active" player row (not a config-write log line or
            // bot/challenging row). The status table has numbers-then-"active"
            // before the quoted name.
            let is_player_row = head.contains(" active ")
                && head.split_whitespace().next().map_or(false, |t| t.chars().all(|c| c.is_ascii_digit()));

            // Reject filenames + paths sometimes quoted in CS2 logs.
            let name_looks_like_file = name.contains('/')
                || name.contains('\\')
                || name.ends_with(".vcfg")
                || name.ends_with(".cfg")
                || name.ends_with("_lastclouded");

            if is_player_row
                && !name.is_empty()
                && !name.contains('\'')
                && !name_looks_like_file
                && !tail.contains('\'')  // multi-quote log line, bail
            {
                return Some((format!("cs2name:{name}"), name.to_string()));
            }
        }
    }

    None
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

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &str = "[EngineServiceManager] ----- Status -----\n[Client] steamid  : [A:1:2065561615:49508] (90284629853861903)\n[Client] ---------players--------\n[Client]   id     time ping loss      state   rate name\n[Client] 65280    04:46   34    0     active 786432 'MaviSlime'\n[Client] 65281    04:46   10    0     active 786432 'sakka'\n[Client] 65282    04:46   15    1     active 786432 'Mr x Praezi'\n[Client] 65283    04:46   16    1     active 786432 'WehrMachtDennSoWas'\n[Client] 65284    04:46   13    0     active 786432 'B1t'\n[Client] 65285    04:45   15    0     active 786432 'GGDelta | Mr Cheng'\n[Client] 65286    04:46   29    0     active 786432 'waeit'\n[Client] 65287    04:46   16    0     active 786432 'Gesus'\n[Client] 65288    04:46   12    0     active 786432 'El ubbi-.'\n[Client] 65289    04:46   23    0     active 786432 'Palle fra Temu'\n[Client] 65535 [NoChan]    0    0 challenging      0 ''\n[Client]   12      BOT    0    0     active      0 'DemoRecorder'\n[Client] #end";

    #[test]
    fn parses_cs2_live_status() {
        let names: Vec<_> = SAMPLE.lines().filter_map(|l| parse_status_line(l).map(|(_, n)| n)).collect();
        println!("parsed: {:?}", names);
        assert_eq!(names.len(), 10, "expected 10 humans, got {}: {:?}", names.len(), names);
    }

    // Console log often contains file-write entries like:
    //   FileSystem: .../convars slot 0 saved - 'cs2_user_convars.vcfg'
    //   FileSystem: 'cfg/cs2_user_keys_0_slot0.vcfg' saved
    // These previously false-matched because the heuristic only required a
    // time-like colon in the head. Ensure they're now rejected.
    const NOISE: &str = "[FileSystem] convars slot 0 saved - 'cs2_user_convars.vcfg'\n[FileSystem] cfg/cs2_user_keys_0_slot0.vcfg saved\n[FileSystem] cfg/cs2_user_keys_0_slot0.vcfg_lastclouded - 'cfg/cs2_user_keys_0_slot0.vcfg_lastclouded'\n[Client] saving 'cs2_user_keys.vcfg'\n[Client] 65280    04:46   34    0     active 786432 'Real Player'";

    #[test]
    fn rejects_file_write_log_lines() {
        let names: Vec<_> = NOISE.lines().filter_map(|l| parse_status_line(l).map(|(_, n)| n)).collect();
        println!("parsed noise: {:?}", names);
        assert_eq!(names, vec!["Real Player".to_string()], "file-write log lines must not parse as players");
    }
}
