use proquint::Quintable;
use std::collections::HashSet;

pub fn get_project_name() -> String {
    // 1. Cargo.toml
    if let Some(name) = std::fs::read_to_string("Cargo.toml")
        .ok()
        .and_then(|content| content.parse::<toml::Value>().ok())
        .and_then(|toml| {
            toml.get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
        })
    {
        return name;
    }
    // 2. pyproject.toml
    if let Some(name) = std::fs::read_to_string("pyproject.toml")
        .ok()
        .and_then(|content| content.parse::<toml::Value>().ok())
        .and_then(|toml| {
            toml.get("project")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
        })
    {
        return name;
    }
    // 3. Directory name
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn hash_seed(name: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

/// A simple 16-bit permutation based on a Feistel network.
fn permute(val: u16, seed: u64) -> u16 {
    let mut l = (val >> 8) as u8;
    let mut r = (val & 0xFF) as u8;
    for i in 0..4 {
        let tmp = l;
        l = r;
        // Simple round function: r = l ^ hash(r, seed, i)
        let mut h = seed ^ (r as u64) ^ (i as u64);
        h = h.wrapping_mul(0x517cc1b727220a95);
        h ^= h >> 32;
        r = tmp ^ (h as u8);
    }
    ((l as u16) << 8) | (r as u16)
}

fn unpermute(val: u16, seed: u64) -> u16 {
    let mut l = (val >> 8) as u8;
    let mut r = (val & 0xFF) as u8;
    for i in (0..4).rev() {
        let tmp = r;
        r = l;
        let mut h = seed ^ (l as u64) ^ (i as u64);
        h = h.wrapping_mul(0x517cc1b727220a95);
        h ^= h >> 32;
        l = tmp ^ (h as u8);
    }
    ((l as u16) << 8) | (r as u16)
}

pub fn generate_next_id(prefix: &str, existing_ids: &HashSet<String>) -> anyhow::Result<String> {
    let project_name = get_project_name();
    let seed = hash_seed(&project_name);

    let mut max_idx = 0u16;
    let mut has_any = false;

    for id in existing_ids {
        if let Some(suffix) = id.strip_prefix(prefix) {
            // proquint crate: from_quint returns (T, usize)
            let (val, _) = proquint::from_quint::<u16>(suffix);
            let idx = unpermute(val, seed);
            if !has_any || idx > max_idx {
                max_idx = idx;
                has_any = true;
            }
        }
    }

    let mut next_idx = if has_any { max_idx.wrapping_add(1) } else { 0 };

    for _ in 0..65536 {
        let candidate_val = permute(next_idx, seed);
        let candidate_quint = candidate_val.to_quint();
        let candidate_id = format!("{}{}", prefix, candidate_quint);

        if !existing_ids.contains(&candidate_id) {
            return Ok(candidate_id);
        }

        next_idx = next_idx.wrapping_add(1);
        if next_idx == (if has_any { max_idx } else { 65535 }) {
            break;
        }
    }

    anyhow::bail!("No available IDs left for prefix {}", prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permutation() {
        let seed = 12345;
        for i in 0..1000 {
            let p = permute(i, seed);
            let u = unpermute(p, seed);
            assert_eq!(i, u);
        }
    }
}
