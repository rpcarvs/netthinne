use std::collections::HashMap;

/// Translates English labels to Norwegian using a dictionary lookup.
pub fn translate(english: &str) -> String {
    let dictionary: HashMap<&str, &str> = HashMap::from([
        ("dog", "hund"),
        ("cat", "katt"),
        ("bird", "fugl"),
        ("fish", "fisk"),
        ("horse", "hest"),
        ("car", "bil"),
        ("bicycle", "sykkel"),
        ("boat", "bat"),
        ("tree", "tre"),
        ("flower", "blomst"),
        ("house", "hus"),
        ("person", "person"),
        ("chair", "stol"),
        ("table", "bord"),
        ("book", "bok"),
        ("phone", "telefon"),
        ("computer", "datamaskin"),
        ("bottle", "flaske"),
        ("cup", "kopp"),
        ("apple", "eple"),
    ]);

    let key = english.to_lowercase();
    match dictionary.get(key.as_str()) {
        Some(norwegian) => norwegian.to_string(),
        None => format!("({})", english),
    }
}
