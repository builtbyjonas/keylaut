//! Character mappings for German umlauts and sharp s.

use std::collections::HashMap;

/// Character mapping manager for Keylaut.
#[derive(Debug, Clone)]
pub struct MappingTable {
    mappings: HashMap<String, String>,
}

impl Default for MappingTable {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl MappingTable {
    /// Creates a mapping table with the default Keylaut mappings.
    ///
    /// Default mappings:
    /// - `ae` -> `ä`
    /// - `oe` -> `ö`
    /// - `ue` -> `ü`
    /// - `ss` -> `ß`
    /// - `Ae` -> `Ä`, `AE` -> `Ä`
    /// - `Oe` -> `Ö`, `OE` -> `Ö`
    /// - `Ue` -> `Ü`, `UE` -> `Ü`
    ///
    /// `SS` -> `ẞ` is explicitly NOT a default mapping per the specification.
    pub fn with_defaults() -> Self {
        let mut map = HashMap::new();

        map.insert("ae".to_string(), "ä".to_string());
        map.insert("oe".to_string(), "ö".to_string());
        map.insert("ue".to_string(), "ü".to_string());
        map.insert("ss".to_string(), "ß".to_string());

        // Capitalized forms
        map.insert("Ae".to_string(), "Ä".to_string());
        map.insert("Oe".to_string(), "Ö".to_string());
        map.insert("Ue".to_string(), "Ü".to_string());

        map.insert("AE".to_string(), "Ä".to_string());
        map.insert("OE".to_string(), "Ö".to_string());
        map.insert("UE".to_string(), "Ü".to_string());

        Self { mappings: map }
    }

    /// Creates a mapping table from a user-provided map, expanding capitalization if missing.
    pub fn from_custom(user_mappings: &HashMap<String, String>) -> Self {
        let mut map = HashMap::new();

        for (k, v) in user_mappings {
            map.insert(k.clone(), v.clone());

            // If key is lowercase 2-char like "ae", automatically provide TitleCase and UPPERCASE
            // unless the user specified them explicitly.
            if k.len() == 2 && k.chars().all(|c| c.is_lowercase()) {
                let mut chars = k.chars();
                let first = chars.next().unwrap();
                let second = chars.next().unwrap();

                // Capitalize replacement if replacement is single char
                let mut val_chars = v.chars();
                if let (Some(val_first), None) = (val_chars.next(), val_chars.next()) {
                    let upper_replacement = val_first.to_uppercase().to_string();

                    let title_key = format!("{}{}", first.to_uppercase(), second);
                    let upper_key = format!("{}{}", first.to_uppercase(), second.to_uppercase());

                    // Special rule: "ss" should NOT default to uppercase "SS" -> "ẞ"
                    if k != "ss" {
                        map.entry(title_key)
                            .or_insert_with(|| upper_replacement.clone());
                        map.entry(upper_key).or_insert_with(|| upper_replacement);
                    }
                }
            }
        }

        Self { mappings: map }
    }

    /// Returns the replacement for an exact matching key sequence, if any.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.mappings.get(key).map(|s| s.as_str())
    }

    /// Returns all registered mappings.
    pub fn mappings(&self) -> &HashMap<String, String> {
        &self.mappings
    }

    /// Replaces occurrences of candidate sequences in a word.
    /// Returns `Some(new_word)` if at least one replacement was made, or `None`.
    pub fn transform_text(&self, text: &str) -> Option<String> {
        let mut result = String::with_capacity(text.len());
        let mut changed = false;
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if let Some(&next_c) = chars.peek() {
                let pair = format!("{}{}", c, next_c);
                if let Some(replacement) = self.get(&pair) {
                    result.push_str(replacement);
                    chars.next(); // consume peeked character
                    changed = true;
                    continue;
                }
            }
            result.push(c);
        }

        if changed {
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mappings() {
        let table = MappingTable::with_defaults();
        assert_eq!(table.get("ae"), Some("ä"));
        assert_eq!(table.get("oe"), Some("ö"));
        assert_eq!(table.get("ue"), Some("ü"));
        assert_eq!(table.get("ss"), Some("ß"));

        assert_eq!(table.get("Ae"), Some("Ä"));
        assert_eq!(table.get("Oe"), Some("Ö"));
        assert_eq!(table.get("Ue"), Some("Ü"));

        assert_eq!(table.get("AE"), Some("Ä"));
        assert_eq!(table.get("OE"), Some("Ö"));
        assert_eq!(table.get("UE"), Some("Ü"));

        // SS -> ẞ must not be enabled by default
        assert_eq!(table.get("SS"), None);
    }

    #[test]
    fn test_transform_text() {
        let table = MappingTable::with_defaults();
        assert_eq!(table.transform_text("schoen"), Some("schön".to_string()));
        assert_eq!(table.transform_text("fuer"), Some("für".to_string()));
        assert_eq!(table.transform_text("groesser"), Some("größer".to_string()));
        assert_eq!(table.transform_text("spaeter"), Some("später".to_string()));
        assert_eq!(table.transform_text("Kaese"), Some("Käse".to_string()));
        assert_eq!(table.transform_text("UEBER"), Some("ÜBER".to_string()));
        assert_eq!(table.transform_text("Ueber"), Some("Über".to_string()));
        assert_eq!(table.transform_text("hallo"), None);
    }
}
