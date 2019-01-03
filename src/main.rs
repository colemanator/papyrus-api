//! # Papyrus
//! A micro-service for querying the bible using approximate string searching.

#![allow(unused)]
extern crate iron;
extern crate router;
extern crate url;

mod bible;
mod search;
mod normalise;

use std::vec::Vec;
use std::process;
use std::time::Instant;
use bible::Bible;
use search::search;
use std::io;
use iron::prelude::*;
use iron::status;
use router::Router;
use url::Url;

fn main() -> io::Result<()> {

    let mut router = Router::new();  
    router.get("/:bible", handler, "bible");

    Iron::new(router).http("localhost:3000").unwrap();

    let bible = match Bible::new("src/data/t_asv.csv") {
        Ok(bible) => bible,
        Err(_) => process::exit(1)
    };

    let verses = bible.get_verses();

    loop {
        // 1. Ask for each string
        println!("\nSearch verses: ");

        // 2. Read input
        let mut query = String::new();
        io::stdin().read_line(&mut query)
            .expect("Failed to read line");

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

fn handler(req: &mut Request) -> IronResult<Response> {
    let url: Url = req.url.clone().into();
        
    let query = url
        .query_pairs()
        .find(|(name, value)| name == "query");

    let (name, value) = match query {
        Some(st) => st,
        None => process::exit(1)
    };

    Ok(Response::with((status::Ok, value.to_string())))
}
