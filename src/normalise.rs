extern crate unicode_normalization;
extern crate caseless;

use unicode_normalization::UnicodeNormalization;
use caseless::Caseless;

pub fn normalise_text(text: &str) -> String {
    caseless::default_case_fold_str(text).nfc().collect()
}
