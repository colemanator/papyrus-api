#![allow(unused)]
extern crate csv;
extern crate unicode_normalization;
extern crate caseless;

mod bible;
mod normalise;

use std::env;
use std::vec::Vec;
use std::cmp;
use std::str;
use std::process;
use std::time::Instant;
use bible::Bible;
use bible::Verse;
use normalise::normalise_text;
use std::io;

 #[derive(Debug)]
 struct Match<'a> {
     verse: &'a Verse<'a>,
     distance: u16
 }


fn main() -> io::Result<()> {

    let bible = match Bible::new("src/data/t_asv.csv") {
        Ok(bible) => bible,
        Err(_) => {}
    };

    let verses = bible.get_verses();

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
        let mut search_chars = search.chars();

        // We want to find the first char that matches and count distance from there
        if let Some(search_char) = search_chars.next() {
            match verse_chars.position(|verse_ch| *verse_ch == search_char) {
                Some(_) => {},
                None => continue 'outer
            }
        }

        let mut distance: u16 = 0;
        for search_char in search_chars {
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
