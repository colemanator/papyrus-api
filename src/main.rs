//! # Papyrus
//! A micro-service for querying the bible using approximate string searching.

#![allow(unused)]
extern crate iron;
extern crate router;
extern crate url;
extern crate persistent;

#[macro_use]
extern crate lazy_static;

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

    let mut router = Router::new();  
    router.get("/:bible", handler, "bible");

    let mut chain = Chain::new(router);
    chain.link(Read::<SharedVerses>::both(shared_verses));

    // Prepare server
    Iron::new(chain).http("localhost:3000").unwrap();

    Ok(())
}

fn handler(req: &mut Request) -> IronResult<Response> {
    let url: Url = req.url.clone().into();
        
    let query = url
        .query_pairs()
        .find(|(name, value)| name == "query");

    let (name, value) = match query {
        Some(st) => st,
        None => process::exit(1)
    };

    let shared_verses = req.get::<Read<SharedVerses>>().unwrap();

    let matches = search(value.to_string(), &shared_verses.verses);

    for m in matches {
        println!("{:?}", m);
    }

    Ok(Response::with((status::Ok, value.to_string())))
}
