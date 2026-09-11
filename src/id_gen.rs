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

// @IMP-votar@ (FROM: @ARC-gamof@)
pub fn generate_ids(
    prefix: &str,
    existing_ids: &HashSet<String>,
    count: usize,
) -> anyhow::Result<Vec<String>> {
    if count == 0 {
        return Ok(Vec::new());
    }
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
    let mut results = Vec::with_capacity(count);
    let mut current_existing = existing_ids.clone();

    for _ in 0..65536 {
        let candidate_val = permute(next_idx, seed);
        let candidate_quint = candidate_val.to_quint();
        let candidate_id = format!("{}{}", prefix, candidate_quint);

        if current_existing.insert(candidate_id.clone()) {
            results.push(candidate_id);
            if results.len() == count {
                return Ok(results);
            }
        }

        next_idx = next_idx.wrapping_add(1);
        if next_idx == (if has_any { max_idx } else { 65535 }) {
            break;
        }
    }

    anyhow::bail!(
        "No available IDs left for prefix {} (generated {} of {} requested)",
        prefix,
        results.len(),
        count
    )
}

pub fn generate_next_id(prefix: &str, existing_ids: &HashSet<String>) -> anyhow::Result<String> {
    generate_ids(prefix, existing_ids, 1).map(|mut v| v.remove(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    // @UT-mozum@ (FROM: @REQ-kitir@)
    #[test]
    fn test_permutation() {
        let seed = 12345;
        for i in 0..1000 {
            let p = permute(i, seed);
            let u = unpermute(p, seed);
            assert_eq!(i, u);
        }
    }

    // @UT-mijom@ (FROM: @REQ-kitir@)
    #[test]
    fn test_permutation_full_range_is_bijective() {
        let seed = 987654321;
        let mut seen = HashSet::new();
        for i in 0..=u16::MAX {
            let p = permute(i, seed);
            assert_eq!(unpermute(p, seed), i);
            assert!(seen.insert(p), "permute produced a duplicate value for {i}");
        }
        assert_eq!(seen.len(), 65536);
    }

    // @UT-vakih@ (FROM: @REQ-kitir@)
    #[test]
    fn test_permutation_different_seeds_diverge() {
        let a = permute(42, 1);
        let b = permute(42, 2);
        assert_ne!(a, b);
    }

    // @UT-vugul@ (FROM: @REQ-kitir@)
    #[test]
    fn test_generate_next_id_empty_set() {
        let existing = HashSet::new();
        let id = generate_next_id("REQ-", &existing).unwrap();
        assert!(id.starts_with("REQ-"));
    }

    // @UT-gamof@ (FROM: @REQ-kitir@)
    #[test]
    fn test_generate_next_id_does_not_collide() {
        let mut existing = HashSet::new();
        for _ in 0..50 {
            let id = generate_next_id("REQ-", &existing).unwrap();
            assert!(
                existing.insert(id.clone()),
                "generated id {id} collided with an existing id"
            );
        }
    }

    // @UT-jafaf@ (FROM: @REQ-kitir@)
    #[test]
    fn test_generate_next_id_ignores_other_prefixes() {
        let mut existing = HashSet::new();
        for _ in 0..10 {
            let id = generate_next_id("OTHER-", &existing).unwrap();
            existing.insert(id);
        }
        // None of the "OTHER-" ids should influence "REQ-" generation,
        // and the two prefixes must not collide with each other.
        let req_id = generate_next_id("REQ-", &existing).unwrap();
        assert!(req_id.starts_with("REQ-"));
        assert!(!existing.contains(&req_id));
    }

    // @UT-bofud@ (FROM: @REQ-kitir@)
    #[test]
    fn test_generate_next_id_exhausted_returns_error() {
        let project_name = get_project_name();
        let seed = hash_seed(&project_name);
        let mut existing = HashSet::new();
        for idx in 0..=u16::MAX {
            let quint = permute(idx, seed).to_quint();
            existing.insert(format!("REQ-{quint}"));
        }
        let result = generate_next_id("REQ-", &existing);
        assert!(result.is_err());
    }

    // @UT-nikag@ (FROM: @REQ-litip@)
    #[test]
    fn test_generate_ids_batch() {
        let mut existing = HashSet::new();
        existing.insert("REQ-lusab".to_string());
        existing.insert("REQ-babad".to_string());

        let batch = generate_ids("REQ-", &existing, 5).unwrap();
        assert_eq!(batch.len(), 5);

        let mut seen = HashSet::new();
        for id in &batch {
            assert!(id.starts_with("REQ-"));
            assert!(
                !existing.contains(id),
                "batch id {id} collided with existing id"
            );
            assert!(seen.insert(id.clone()), "duplicate id {id} within batch");
        }

        let empty = generate_ids("REQ-", &existing, 0).unwrap();
        assert!(empty.is_empty());
    }
}
