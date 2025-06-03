use std::{
    str::FromStr,
    sync::{Arc, LazyLock},
};

use parse_display::{Display, helpers::regex::Regex};
use serde::{Deserialize, Serialize};

use crate::books::BookError;

static VALID_TEXT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{L}\p{N}\p{P}\s]+$").unwrap());

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
#[serde(try_from = "String", into = "String")]
pub struct ValidatedStr {
    value: Arc<str>,
}

impl ValidatedStr {
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl FromStr for ValidatedStr {
    type Err = BookError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            Err(BookError::EmptyString)?;
        }

        if !VALID_TEXT_REGEX.is_match(trimmed) {
            Err(BookError::InvalidCharacters)?;
        }

        let validated_str = Self {
            value: Arc::from(trimmed),
        };

        Ok(validated_str)
    }
}

impl TryFrom<String> for ValidatedStr {
    type Error = BookError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<ValidatedStr> for String {
    fn from(value: ValidatedStr) -> Self {
        value.value.to_string()
    }
}

static VALID_ISBN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:ISBN[- ]?(?:10|13)?:? )?(?:\d{3}[- ]?)?\d{1,5}[- ]?\d{1,7}[- ]?\d{1,7}[- ]?[0-9X]$",
    )
    .unwrap()
});

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
#[serde(try_from = "String", into = "String")]
pub struct Isbn {
    value: Arc<str>,
}

impl Isbn {
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Retorna la forma canonica del ISBN: unicamente el numero sin guiones ni espacios
    ///
    /// # Example
    /// ```
    /// use library_api::books::books_domain::Isbn;
    /// use std::str::FromStr;
    ///
    /// let isbn = Isbn::from_str("ISBN-13 978-0-306-40615-7").unwrap();
    /// assert_eq!(isbn.canonical(), "9780306406157");
    /// ```
    pub fn canonical(&self) -> String {
        self.value
            .replace("ISBN-10", "")
            .replace("ISBN-13", "")
            .replace("ISBN", "")
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == 'X')
            .collect()
    }
}

impl FromStr for Isbn {
    type Err = BookError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if !VALID_ISBN_REGEX.is_match(trimmed) {
            Err(BookError::InvalidIsbn)?;
        }

        let isbn = Self {
            value: Arc::from(trimmed),
        };

        Ok(isbn)
    }
}

impl TryFrom<String> for Isbn {
    type Error = BookError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<Isbn> for String {
    fn from(value: Isbn) -> Self {
        value.value.to_string()
    }
}
