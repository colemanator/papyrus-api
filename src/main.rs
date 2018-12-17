#![allow(unused)]
extern crate csv;
extern crate unicode_normalization;
extern crate caseless;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::env;
use std::vec::Vec;
use csv::Reader;
use csv::StringRecord;
use csv::Error;
use std::cmp;
use std::str;
use std::process;
use std::time::Instant;
use unicode_normalization::UnicodeNormalization;
use caseless::Caseless;

#[derive(Debug)]
 struct Verse<'a> {
    book: u8,
    chapter: u8,
    verse: u8,
    read_text: &'a[char],
    search_text: &'a[char]
 }

 #[derive(Debug)]
 struct Match<'a> {
     verse: &'a Verse<'a>,
     distance: u16
 }

#[derive(Debug)]
struct Bible {
    read_chars: Vec<char>,
    search_chars: Vec<char>,
    verse_selects: Vec<VerseSelect>
}

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

fn main() -> io::Result<()> {

    let bible = match load_bible() {
        Ok(bible) => bible,
        Err(_) => process::exit(1)
    };

    let verses = get_verses(&bible);

    loop {
        // 1. Ask for each string
        println!("\nSearch verses: ");

        // 2. Read input
        let mut query = String::new();
        io::stdin().read_line(&mut query)
            .expect("Failed to read line");

        // 3. Remove new line char and break search into chars
        let query = normalise_text(&query).trim().to_string();

        // 5. If no input was given exit the program
        if query.chars().count() == 0  {
            process::exit(1);
        }

        let now = Instant::now();
        let matches = search(query, &verses);
        println!("{:?}", now.elapsed());

        for m in matches {
            println!("{:?}\n", m);
        }
    }
}

fn search<'a>(search: String, verses: &'a Vec<Verse>) -> Vec<Match<'a>> {
    let mut matches: Vec<Match> = Vec::new();

    // 6. loop through each verse and find best matches
    'outer: for verse in verses {
        let mut verse_chars = verse.search_text.iter();
        let mut distance: u16 = 0;
        for search_char in search.chars() {
            let index = match verse_chars.position(|verse_ch| *verse_ch == search_char) {
                Some(index) => index as u16,
                None => continue 'outer
            };

            distance = index + distance;
        }

        // Only store the match a better match than the current worst match
        if let Some(lower_bound_match) = matches.last() {
            if (lower_bound_match.distance < distance) {
                continue;
            }
        }
        
        matches.push(Match { 
            verse: &verse,
            distance
        });

        matches = top_matches(matches, 10);
    }

    matches
}

fn top_matches(mut matches: Vec<Match>, limit: u8) -> Vec<Match> {
    matches.sort_unstable_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    matches.truncate(limit as usize);

    matches
}

fn normalise_text(text: &str) -> String {
    caseless::default_case_fold_str(text).nfc().collect()
}

fn load_bible() -> Result<Bible, Error>{
    let mut rdr = Reader::from_path("src/data/t_asv.csv")?;
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
        let book = as_int(&record, 1);
        let chapter = as_int(&record, 2);
        let verse = as_int(&record, 3);
        let text = as_string(&record, 4);
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

fn as_string(record: &StringRecord, index: usize) -> String {
    let text = match record.get(index) {
        Some(text) => text,
        None => process::exit(1)
    };

    text.to_string()
}

fn get_verses<'a>(bible: &'a Bible) -> Vec<Verse<'a>> {
    let mut verses = Vec::new();

    for verse_select in bible.verse_selects.iter() {
        verses.push(Verse{
            read_text: &bible.read_chars[verse_select.read_position as usize..(verse_select.read_position + verse_select.read_length as u32) as usize],
            search_text: &bible.search_chars[verse_select.search_position as usize..(verse_select.search_position + verse_select.search_length as u32) as usize],
            book: verse_select.book,
            chapter: verse_select.chapter,
            verse: verse_select.verse
        });
    }

    verses
}