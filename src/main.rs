#![allow(unused)]
extern crate csv;
extern crate num_cpus;
extern crate scoped_threadpool;
extern crate elapsed;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::env;
use std::vec::Vec;
use std::slice::Chunks;
use csv::Reader;
use std::cmp;
use std::str;
use std::process;
use scoped_threadpool::Pool;
use elapsed::measure_time;

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
     matched_indexes: Vec<u16>,
     score: u16
 }

 #[derive(Debug)]
 struct Job<'b> {
    chunk: &'b [Verse],
    search: String,
    matches: Vec<Match<'b>>
 }

fn main() -> io::Result<()> {
    let verses = match load_verses() {
        Ok(verses) => verses,
        Err(e) => process::exit(1)
    };

    // Prepare the thread pool with n threads where n is the number of logical cores
    let mut pool = Pool::new(1 as u32);

    loop {
        // 1. Ask for each string
        println!("\nSearch verses: ");

        // 2. Read input
        let mut search = String::new();
        io::stdin().read_line(&mut search)
            .expect("Failed to read line");

        // 3. Remove new line char and break search into chars
        let search = search.trim().to_lowercase();

        // 5. If no input was given exit the program
        if search.chars().count() == 0  {
            process::exit(1);
        }

        let (elapsed, sum) = measure_time(|| {
            let mut jobs = make_jobs(search, &verses, &pool);

            pool.scoped(|scope| {
                for job in &mut jobs {
                    scope.execute(move || {
                        process_job(job)
                    });
                }
            });

            let top_matches = top_job_matches(jobs, 10);

            for top_match in top_matches {
                println!("{:?}\n", top_match);
            }
        });

        println!("{:?}\n", elapsed);

        // 7. top return verse location
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
            Verse { book, chapter, verse, text: text.to_string().to_lowercase().trim().to_string() }
        );
    }

    Ok(verses)
}

fn make_jobs<'b>(search: String, verses: &'b Vec<Verse>, pool: &Pool) -> Vec<Job<'b>> {
    // calculate the size of each chunk of work we will give to each thread in the pool
        let verse_chunk_size = ((verses.len() as f32) / (pool.thread_count() as f32)).ceil() as u16;

        // Create a job for each thread
        return verses
            .chunks(verse_chunk_size as usize)
            .map(|chunk| Job {
                chunk,
                matches: Vec::new(),
                search: search.clone()
            })
            .collect();
}

fn process_job(job: &mut Job) {
    // 6. loop through each verse and find best matches
    'outer: for verse in job.chunk {
        let mut matched_indexes: Vec<u16> = Vec::new();
        let mut verse_chars = verse.text.chars().enumerate();

        for search_char in job.search.chars() {
            match verse_chars.find(|(i, verse_ch)| *verse_ch == search_char) {
                Some((i, verse_ch)) => matched_indexes.push(i as u16),
                None => continue 'outer
            }
        }

        let score: u16 = match score(&matched_indexes) {
            Ok(score) => score,
            Err(e) => continue
        };

        // Only store the match a better match than the current worst match
        if let Some(lower_bound_match) = job.matches.last() {
            if (lower_bound_match.score < score) {
                continue;
            }
        }
        
        job.matches.push(Match { 
            verse: verse,
            score,
            matched_indexes
        });

        job.matches.sort_unstable_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
        job.matches.truncate(10);
    }
}

fn top_job_matches<'a>(jobs: Vec<Job<'a>>, limit: u8) -> Vec<Match<'a>> {
    let mut top_matches: Vec<Match> = jobs
        .into_iter()
        .flat_map(|job| job.matches)
        .collect();

    top_matches.sort_unstable_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
    top_matches.truncate(limit as usize);

    top_matches
}

fn score(matched_indexes: &Vec<u16>) -> Result<u16, &'static str> {
    let first_index = match matched_indexes.first() {
        Some(index) => index,
        None => return Err("must contain at least one index")
    };

    let last_index = match matched_indexes.last() {
        Some(index) => index,
        None => return Err("must contain at least one index")
    };

    Ok(last_index - first_index)
}