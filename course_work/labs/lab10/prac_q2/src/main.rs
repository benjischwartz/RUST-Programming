use std::fs;
use std::env;
use std::io::{self, BufRead};
use std::error::Error;
use std::collections::HashMap;

// NOTE: You *may not* change the names or types of the members of this struct.
//       You may only add lifetime-relevant syntax.
#[derive(Debug)]
pub struct SearchResult<'a, 'b> {
    pub matches: Vec<&'a str>,
    pub contains: &'b str
}

/// Returns a [`SearchResult`] struct, where the matches vec is
/// a vector of every sentence that contains `contains`.
///
/// A sentence is defined as a slice of an `&str` which is the first
/// character of the string, or the first non-space character after
/// a full-stop (`.`), all the way until the last non-space character
/// before a full-stop or the end of the string.
///
/// For example, In the string "Hello. I am Tom . Goodbye", the three
/// sentences are "Hello", "I am Tom" and "Goodbye"
fn find_sentences_containing<'a, 'b>(text: &'a str, contains: &'b str) -> SearchResult<'a, 'b> {
    let mut sentences = Vec::new();
    let mut start = None;
    let mut end = None;
    let mut in_sentence = false;

    for (i, c) in text.chars().enumerate() {
        if c == '.' {
            if let Some(_) = start {
                end = Some(i);
                in_sentence = false;
            }
        } else {
            if !in_sentence {
                start = Some(i);
                in_sentence = true;
            }
        }
        if let (Some(start_idx), Some(end_idx)) = (start, end) {
            let sentence = text[start_idx..end_idx].trim();
            if !sentence.is_empty() && sentence.contains(contains) {
                sentences.push(sentence);
            }
            start = None;
            end = None;
        }
    }

    if let Some(start_idx) = start {
        let sentence = &text[start_idx..].trim();
        if !sentence.is_empty() {
            sentences.push(sentence);
        }
    }
    SearchResult{matches: sentences, contains}
}

/// Given a vec of [`SearchResult`]s, return a hashmap, which lists how many
/// times each sentence occurred in the search results.
fn count_sentence_matches<'a, 'b>(searches: Vec<SearchResult<'a, 'b>>) -> HashMap<&'a str, i32> {
    let mut res: HashMap<&'a str, i32> = HashMap::new();
    for search in searches {
        for search_match in &search.matches {
            *res.entry(search_match).or_insert(0) += 1;
        }
    }
    res
}


/////////// DO NOT CHANGE BELOW HERE ///////////

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let text = fs::read_to_string(file_path)?;

    let mut sentence_matches = {
        let mut found = vec![];

        let stdin = io::stdin();
        let matches = stdin.lock().lines().map(|l| l.unwrap()).collect::<Vec<_>>();
        for line in matches.iter() {
            let search_result = find_sentences_containing(&text, line);
            println!("Found {} results for '{}'.", search_result.matches.len(), search_result.contains);
            found.push(search_result);
        }

        count_sentence_matches(found).into_iter().collect::<Vec<_>>()
    };
    sentence_matches.sort();

    for (key, value) in sentence_matches {
        println!("'{}' occured {} times.", key, value);
    }

    Ok(())
}
