use regex::Regex;
use std::convert;
use std::error;
use std::fmt::{self, Display};

#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    value: String,
    datasource: String,
    dep_name: String,
    id: String,
}

impl Entry {
    pub fn new(value: String, datasource: String, dep_name: String, id: String) -> Entry {
        Entry {
            value,
            datasource,
            dep_name,
            id,
        }
    }
    pub fn has_id(&self, id: &str) -> bool {
        id == self.id
    }
    pub fn get_id(&self) -> String {
        self.id.to_string()
    }
    pub fn get_value(&self) -> String {
        self.value.to_string()
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "straight={} depName={} datasource={} value={}",
            self.id, self.dep_name, self.datasource, self.value
        )
    }
}

#[derive(Debug)]
pub struct ParseEntryError(String);

impl error::Error for ParseEntryError {}
impl Display for ParseEntryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseEntryError({})", self.0)
    }
}

impl convert::TryFrom<String> for Entry {
    type Error = ParseEntryError;
    fn try_from(s: String) -> Result<Entry, Self::Error> {
        let re = Regex::new(
            r"straight=(?<id>.+) depName=(?<dep_name>.+) datasource=(?<datasource>.+) value=(?<value>.+)",
        ).unwrap();
        if let Some(cs) = re.captures(&s) {
            let id = &cs["id"];
            let dep_name = &cs["dep_name"];
            let datasource = &cs["datasource"];
            let value = &cs["value"];
            Ok(Entry::new(
                value.to_string(),
                datasource.to_string(),
                dep_name.to_string(),
                id.to_string(),
            ))
        } else {
            Err(ParseEntryError(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! test_conversion {
        ($name:ident, $text:expr, $entry:expr) => {
            #[test]
            fn $name() {
                let got = Entry::try_from($text.to_string()).unwrap();
                assert_eq!($entry, got);
                let got = $entry.to_string();
                assert_eq!($text.to_string(), got);
            }
        };
    }

    test_conversion!(
        test_conversion_entry,
        "straight=ID depName=DEPNAME datasource=DATASOURCE value=VALUE",
        Entry::new(
            "VALUE".to_string(),
            "DATASOURCE".to_string(),
            "DEPNAME".to_string(),
            "ID".to_string()
        )
    );
}
