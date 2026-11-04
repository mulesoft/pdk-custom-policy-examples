use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PiiType {
    Ssn,
    Email,
    CreditCard,
    PhoneNumber,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiMatch {
    pub pii_type: PiiType,
    // Never serialize the matched value: it is the PII itself and would leak
    // into logs and response bodies.
    #[serde(skip_serializing, default)]
    pub value: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Error, Debug)]
pub enum PiiError {
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(String),
}

pub trait PiiDetector {
    fn detect(&self, text: &str) -> Result<Vec<PiiMatch>, PiiError>;
}

pub struct RegexPiiDetector {
    patterns: Vec<(PiiType, Regex)>,
}

impl RegexPiiDetector {
    pub fn new() -> Result<Self, PiiError> {
        let patterns: Vec<(PiiType, Regex)> = vec![
            (
                // SSN pattern (XXX-XX-XXXX)
                PiiType::Ssn,
                Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")
                    .map_err(|e| PiiError::InvalidRegex(e.to_string()))?,
            ),
            (
                // Email pattern
                PiiType::Email,
                Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")
                    .map_err(|e| PiiError::InvalidRegex(e.to_string()))?,
            ),
            (
                // Credit Card pattern (supports major card types)
                PiiType::CreditCard,
                Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b")
                    .map_err(|e| PiiError::InvalidRegex(e.to_string()))?,
            ),
            (
                // US Phone Number pattern
                PiiType::PhoneNumber,
                Regex::new(r"\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b")
                    .map_err(|e| PiiError::InvalidRegex(e.to_string()))?,
            ),
        ];

        Ok(Self { patterns })
    }
}

impl PiiDetector for RegexPiiDetector {
    fn detect(&self, text: &str) -> Result<Vec<PiiMatch>, PiiError> {
        let mut matches = Vec::new();

        for (pii_type, pattern) in &self.patterns {
            for mat in pattern.find_iter(text) {
                matches.push(PiiMatch {
                    pii_type: pii_type.clone(),
                    value: mat.as_str().to_string(),
                    start: mat.start(),
                    end: mat.end(),
                });
            }
        }

        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_matches_do_not_leak_pii_values() {
        let detector = RegexPiiDetector::new().unwrap();
        let text = "My SSN is 123-45-6789 and my email is john.doe@example.com";

        let matches = detector.detect(text).unwrap();
        let serialized = serde_json::to_string_pretty(&matches).unwrap();

        assert!(!serialized.contains("123-45-6789"), "{}", serialized);
        assert!(!serialized.contains("john.doe@example.com"), "{}", serialized);
        // Type and position are still reported
        assert!(serialized.contains("Ssn"));
        assert!(serialized.contains("\"start\""));
    }

    #[test]
    fn test_regex_detector() {
        let detector = RegexPiiDetector::new().unwrap();
        let text = "My SSN is 123-45-6789 and my email is john.doe@example.com";

        let matches = detector.detect(text).unwrap();
        assert_eq!(matches.len(), 2);

        let ssn_match = matches.iter().find(|m| m.pii_type == PiiType::Ssn).unwrap();
        assert_eq!(ssn_match.value, "123-45-6789");

        let email_match = matches
            .iter()
            .find(|m| m.pii_type == PiiType::Email)
            .unwrap();
        assert_eq!(email_match.value, "john.doe@example.com");
    }
}
