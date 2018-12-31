//! # Bible
//! 
//! This module contains the bible struct and it accompanying functions

extern crate csv;

use normalise::normalise_text;
use csv::Reader;
use csv::StringRecord;
use csv::Error;
use std::process;
use std::path::Path;

// Contains verses as a flat vector of chars with verse_selects which mark each verse
#[derive(Debug)]
pub struct Bible {
    read_chars: Vec<char>,
    search_chars: Vec<char>,
    verse_selects: Vec<VerseSelect>
}

// Used by the bible struct to identify verses in a flat vector of chars
#[derive(Debug)]
struct VerseSelect {
    read_position: u32,
    read_length: u16,
    search_position: u32,
    search_length: u16,
    verse: u8,
    chapter: u8,
    book: u8
}

// Represents a bible verse
#[derive(Debug)]
 pub struct Verse<'a> {
    pub book: u8,
    pub chapter: u8,
    pub verse: u8,
    pub read_text: &'a[char],
    pub search_text: &'a[char]
 }

 impl Bible {

    /// Create a bible struct from a CSV
    pub fn new(path: &str) -> Result<Bible, Error>{
        let mut rdr = Reader::from_path(path)?;
        let headers = rdr.headers()?.clone();

        let mut bible = Bible{
            search_chars: Vec::new(),
            read_chars: Vec::new(),
            verse_selects: Vec::new()
        };

        let mut read_text_index = bible.read_chars.len();
        let mut search_text_index = bible.search_chars.len();

        for result in rdr.records() {
            let record = result?;

            // Get each property of the record
            let book = Bible::as_int(&record, 1);
            let chapter = Bible::as_int(&record, 2);
            let verse = Bible::as_int(&record, 3);
            let text = Bible::as_string(&record, 4);
            let search_text = normalise_text(&text.to_string());

            // Adding chars to the vectors
            bible.read_chars.extend(text.chars());
            bible.search_chars.extend(search_text.chars());

            // Make a ref for this verse
            bible.verse_selects.push(VerseSelect{
                read_position: read_text_index as u32,
                read_length: text.len() as u16,
                search_position: search_text_index as u32,
                search_length: search_text.len() as u16,
                book,
                chapter,
                verse
            });

            read_text_index = bible.read_chars.len();
            search_text_index = bible.search_chars.len();
        }

        Ok(bible)
    }

    /// Get the verses from the bible
    pub fn get_verses<'a>(&'a self) -> Vec<Verse<'a>> {
        let mut verses = Vec::new();

        for verse_select in self.verse_selects.iter() {
            verses.push(Verse{
                read_text: &self.read_chars[verse_select.read_position as usize..(verse_select.read_position + verse_select.read_length as u32) as usize],
                search_text: &self.search_chars[verse_select.search_position as usize..(verse_select.search_position + verse_select.search_length as u32) as usize],
                book: verse_select.book,
                chapter: verse_select.chapter,
                verse: verse_select.verse
            });
        }

        verses
    }

    /// Read a record from the CSV and attempt to cast to a u8
    fn as_int(record: &StringRecord, index: usize) -> u8 {
        match record.get(index) {
            Some(st) => {
                match st.trim().parse() {
                    Ok(integer) => integer,
                    Err(_) => process::exit(1)
                }
            }
            None => process::exit(1)
        }
    }

    /// Read a record from a CSV as a string
    fn as_string(record: &StringRecord, index: usize) -> String {
        let text = match record.get(index) {
            Some(text) => text,
            None => process::exit(1)
        };

        text.to_string()
    }
 }