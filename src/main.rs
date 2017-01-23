extern crate cpython;
extern crate crypto;
extern crate env_logger;
extern crate iron;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate openssl;
extern crate petgraph;
extern crate rustc_serialize;
extern crate regex;
extern crate router;

pub mod core;
pub mod rori_utils;

use core::Server;
use core::API;
use core::words_manager::WordsManager;
use std::thread;

#[allow(dead_code)]
fn main() {
    let wm = WordsManager::new(String::from("wordsclassification"));
    wm.get_words_from_category(String::from("color"));
    println!("{:?}", wm.is_word_in_category(String::from("red"), String::from("color")));
    println!("{:?}", wm.is_word_in_category(String::from("red2"), String::from("color")));

    // Init logging
    env_logger::init().unwrap();

    // Launch API
    thread::spawn(move || {
        let mut api = API::new("config_server.json");
        api.start();
    });
    // Launch RORI server
    let mut server = Server::new("config_server.json");
    server.start();
}
