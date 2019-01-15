//! # Search
//! 
//! The module contains the functions and structs needed for performing the search
//! The search is an optimized fuzzy or appropriate string search 

use normalise::normalise_text;
use std::vec::Vec;
use bible::Verse;

/// Represents a match
#[derive(Debug)]
 pub struct Match<'a> {
     pub verse: &'a Verse<'a>,
     pub distance: u16
 }

/// Find the top 10 matches in the given verses
/// 
/// # Arguments
/// 
/// * `query` - The string query
/// * `verses` - A Vector of verses
/// 
/// # Example
/// let matches: Match = search(query, verses);
/// 
pub fn search<'a>(query: String, verses: &'a Vec<Verse>) -> Vec<Match<'a>> {
    let mut matches: Vec<Match> = Vec::new();

    // 3. Remove new line char and break query into chars
    let query = normalise_text(&query).trim().to_string();

    // loop through each verse and find best matches
    'outer: for verse in verses {
        let mut verse_chars = verse.search_text.iter();
        let mut query_chars = query.chars();

        // We want to find the first char that matches and count distance from there
        if let Some(query_char) = query_chars.next() {
            match verse_chars.position(|verse_ch| *verse_ch == query_char) {
                Some(_) => {},
                None => continue 'outer
            }
        }

        // For each of the remaining chars try and find a match and record the distance between matches
        let mut distance: u16 = 0;
        for query_char in query_chars {
            let index = match verse_chars.position(|verse_ch| *verse_ch == query_char) {
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

/// Sort matches and keep only the top 10
/// 
/// # Arguments
/// 
/// * `matches` - A vector of matches which we want to get the top matches from
/// * `limit` - The number of top matches to return
fn top_matches(mut matches: Vec<Match>, limit: u8) -> Vec<Match> {
    matches.sort_unstable_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    matches.truncate(limit as usize);

    matches
}