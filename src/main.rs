#![allow(unused)]
extern crate csv;
extern crate unicode_normalization;
extern crate caseless;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::env;
use std::vec::Vec;
use csv::Reader;
use std::cmp;
use std::str;
use std::process;
use std::time::Instant;
use unicode_normalization::UnicodeNormalization;
use caseless::Caseless;

#[derive(Debug)]
 struct Verse {
    book: u8,
    chapter: u8,
    verse: u8,
    text: String
 }

 #[derive(Debug)]
 struct Match<'a> {
     verse: &'a Verse,
     distance: u16
 }

fn main() -> io::Result<()> {
    let verses = match load_verses() {
        Ok(verses) => verses,
        Err(_) => process::exit(1)
    };

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

fn load_verses() -> Result<Vec<Verse>, csv::Error> {
    let mut rdr = Reader::from_path("src/data/t_asv.csv")?;
    let headers = rdr.headers()?.clone();

    let mut verses = Vec::new();

    for result in rdr.records() {
        let record = result?;

        let book: &str = match record.get(1) {
            Some(x) => x,
            None => continue
        };

        let book: u8 = match book.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        let chapter: &str = match record.get(2) {
            Some(x) => x,
            None => continue
        };

        let chapter: u8 = match chapter.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        let verse: &str = match record.get(3) {
            Some(x) => x,
            None => continue
        };

        let verse: u8 = match verse.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        let text: &str = match record.get(4) {
            Some(x) => x,
            None => continue
        };

        verses.push(
            Verse { 
                book,
                chapter,
                verse,
                text: normalise_text(text).trim().to_string()
            }
        );
    }

    Ok(verses)
}

fn search<'a>(search: String, verses: &'a Vec<Verse>) -> Vec<Match<'a>> {
    let mut matches: Vec<Match> = Vec::new();

    // 6. loop through each verse and find best matches
    'outer: for verse in verses {
        let mut verse_chars = verse.text.chars();
        let mut distance: u16 = 0;
        for search_char in search.chars() {
            let index = match verse_chars.position(|verse_ch| verse_ch == search_char) {
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