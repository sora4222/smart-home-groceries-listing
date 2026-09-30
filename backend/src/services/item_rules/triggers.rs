//! Tidying the trigger phrases a rule matches item names against.

use crate::models::schemas::MAX_TRIGGERS;

/// Tidies a trigger list for storage: collapses runs of whitespace, drops
/// blanks, removes case-insensitive repeats keeping the first spelling, and
/// caps the count at [`MAX_TRIGGERS`].
///
/// Case is kept as typed so the settings page shows what the user wrote;
/// matching ignores it anyway.
pub fn clean(triggers: &[String]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut cleaned: Vec<String> = Vec::new();
    for trigger in triggers {
        let trigger = trigger.split_whitespace().collect::<Vec<_>>().join(" ");
        if trigger.is_empty() || cleaned.len() >= MAX_TRIGGERS {
            continue;
        }
        let key = trigger.to_lowercase();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        cleaned.push(trigger);
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::clean;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn whitespace_is_collapsed_and_blanks_dropped() {
        assert_eq!(
            clean(&strings(&["  toilet \t paper ", "   "])),
            vec!["toilet paper"]
        );
    }

    #[test]
    fn a_repeat_keeps_its_first_spelling() {
        assert_eq!(
            clean(&strings(&["Toilet Paper", "toilet  paper", "loo roll"])),
            vec!["Toilet Paper", "loo roll"]
        );
    }

    #[test]
    fn triggers_are_capped_at_the_column_limit() {
        let many: Vec<String> = (0..15).map(|n| format!("item {n}")).collect();
        assert_eq!(clean(&many).len(), 10);
    }
}
