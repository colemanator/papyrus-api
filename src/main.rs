//! # Papyrus
//! A micro-service for querying the bible using approximate string searching.

#![allow(unused)]
extern crate iron;
extern crate router;
extern crate url;
extern crate persistent;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

mod bible;
mod search;
mod normalise;

use std::vec::Vec;
use std::process;
use std::time::Instant;
use bible::Bible;
use bible::Verse;
use search::search;
use std::io;
use iron::prelude::*;
use iron::status;
use iron::{Iron, Chain};
use router::Router;
use url::Url;
use persistent::Read;
use std::sync::Arc;
use iron::{typemap, AfterMiddleware, BeforeMiddleware};

#[derive(Debug)]
struct SharedVerses<'a> {
    verses: Vec<Verse<'a>>
}

#[derive(Serialize, Debug)]
struct MatchedVerse {
    score: u16,
    text: String,
    book: u8,
    chapter: u8,
    verse: u8
}

/// Allows storing `SharedVerses` in Iron's persistent data structure.
/// This needs to be static
impl ::iron::typemap::Key for SharedVerses<'static> {
    type Value = SharedVerses<'static>;
}

fn main() -> io::Result<()> {
    // lazy Load bible as static
    lazy_static! {
        static ref BIBLE: Bible = match Bible::new("src/data/t_asv.csv") {
            Ok(b) => b,
            Err(_) => process::exit(1)
        };
    }

    // Get verses from bible
    let shared_verses = SharedVerses { verses: BIBLE.get_verses() };

    // Setup the endpoint
    let mut router = Router::new();  
    router.get("/:bible", handler, "bible");

    // Add router to middleware chain as well as shared verses
    let mut chain = Chain::new(router);
    chain.link(Read::<SharedVerses>::both(shared_verses));

    // Start server
    Iron::new(chain).http("localhost:3000").unwrap();

    Ok(())
}

/// Handles incoming requests to a specific route
fn handler(req: &mut Request) -> IronResult<Response> {
    let query = get_query(req);

    let shared_verses = req.get::<Read<SharedVerses>>().unwrap();

    let matches = search(query, &shared_verses.verses);

    let search_matches: Vec<MatchedVerse> = matches
        .iter()
        .map(|mat| MatchedVerse {
            score: mat.distance,
            text: mat.verse.read_text.iter().collect(),
            book: mat.verse.book,
            chapter: mat.verse.chapter,
            verse: mat.verse.verse
        })
        .collect();

    let match_json = match serde_json::to_string(&search_matches) {
        Ok(json) => json,
        Err(_) => process::exit(1)
    };

    Ok(Response::with((status::Ok, match_json)))
}

/// Find and return the query param value with the name `query`
fn get_query(req: &mut Request) -> String {
    let url: Url = req.url.clone().into();
        
    let query = url
        .query_pairs()
        .find(|(name, value)| name == "query");

    let (name, value) = match query {
        Some(st) => st,
        None => process::exit(1)
    };

    value.to_string()
}
