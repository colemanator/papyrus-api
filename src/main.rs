//! # Papyrus
//! A micro-service for querying the bible using approximate string searching.

#![allow(unused)]
extern crate iron;
extern crate router;
extern crate url;
extern crate persistent;
extern crate serde;
extern crate serde_json;
extern crate dotenv;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

mod bible;
mod search;
mod normalise;

use std::vec::Vec;
use std::env;
use std::process;
use std::time::Instant;
use bible::Bible;
use bible::Verse;
use search::search;
use search::Match;
use std::io;
use iron::prelude::*;
use iron::status;
use iron::{Iron, Chain};
use router::Router;
use url::Url;
use persistent::Read;
use std::sync::Arc;
use iron::{typemap, AfterMiddleware, BeforeMiddleware};
use iron::headers::{AccessControlAllowOrigin, ContentType};
use iron::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use dotenv::dotenv;
use serde_json::Result;

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

// After middleware for headers
struct Headers;
impl AfterMiddleware for Headers {
    /// Here we set the standard headers for our API
    fn after(&self, req: &mut Request, mut res: Response) -> IronResult<Response> {
        res.headers.set(
            AccessControlAllowOrigin::Value(env::var("APP_ALLOWED_ORIGIN").unwrap().to_owned())
        );
        res.headers.set(
            ContentType(Mime(TopLevel::Application, SubLevel::Json,vec![(Attr::Charset, Value::Utf8)]))
        );

        Ok(res)
    }
}

/// Allows storing `SharedVerses` in Iron's persistent data structure.
/// This needs to be static
impl ::iron::typemap::Key for SharedVerses<'static> {
    type Value = SharedVerses<'static>;
}

fn main() -> io::Result<()> {
    // Load environment variables from .env
    dotenv().ok();

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
    router.get("/:verses", handler, "verses");

    // Add router to middleware chain as well as shared verses
    let mut chain = Chain::new(router);
    chain.link(Read::<SharedVerses>::both(shared_verses));
    chain.link_after(Headers);

    // Start server
    Iron::new(chain).http(env::var("APP_SOCKET_ADDRESS").unwrap()).unwrap();

    Ok(())
}

/// Handles incoming requests to a specific route
fn handler(req: &mut Request) -> IronResult<Response> {
    // Get the query and search verses
    let query = match get_query(req) {
        Some(query) => query,
        None => {
            return Ok(Response::with((status::BadRequest, "You must provide a query")));
        }
    };

    // Perform the search on shared verses
    let shared_verses = req.get::<Read<SharedVerses>>().unwrap();
    let matches = search(query, &shared_verses.verses);

    // Convert the results to JSON
    let match_json = match match_to_json(matches) {
        Ok(json) => json,
        Err(_) => {
            return Ok(Response::with((status::InternalServerError, "Unable to serialize to JSON;")));
        }
    };

    // Create response
    Ok(Response::with((status::Ok, match_json)))
}

/// Find and return the query param value with the name `query`
fn get_query(req: &mut Request) -> Option<String> {
    let url: Url = req.url.clone().into();
        
    let query = url
        .query_pairs()
        .find(|(name, value)| name == "query");

    let (name, value) = match query {
        Some(st) => st,
        None => {
            return None;
        }
    };

    Some(value.to_string())
}

fn match_to_json(matches: Vec<Match>) -> Result<String> {
    // Convert to response structure
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

    serde_json::to_string(&search_matches)
}
