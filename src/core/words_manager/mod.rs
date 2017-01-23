use petgraph::graphmap::DiGraphMap;
use std::fs::File;
use std::io::prelude::*;
use std::mem;

pub struct WordsManager {
    pub graph: DiGraphMap<&'static str, i64>,
}

fn string_to_static_str(s: String) -> &'static str {
    unsafe {
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

impl WordsManager {
    pub fn new(word_file_path: String) -> WordsManager {
        let mut file = File::open(word_file_path)
            .ok()
            .expect("Can't find modules for this type");
        let mut to_parse = String::new();
        file.read_to_string(&mut to_parse)
            .ok()
            .expect("failed to read module list");
        WordsManager { graph: WordsManager::build_graph(to_parse) }
    }

    pub fn build_graph(to_parse: String) -> DiGraphMap<&'static str, i64> {
        let mut final_graph = DiGraphMap::new();
        let to_parse: &'static str = string_to_static_str(to_parse);
        let lines = to_parse.split("\n");
        for line in lines {
            if line.contains(":") {
                let mut words = line.split(":");
                if words.clone().count() == 2 {
                    let child_node_key = words.next().unwrap_or("");
                    let parent_node_key = words.next().unwrap_or("");
                    final_graph.add_node(child_node_key);
                    final_graph.add_node(parent_node_key);
                    final_graph.add_edge(parent_node_key, child_node_key, 1);
                }
            } // else ignore line
        }
        final_graph
    }

    pub fn get_words_from_category(&self, category: String) -> Vec<String> {
        let mut words_list = Vec::new();
        let mut to_develop = Vec::new();
        to_develop.push(string_to_static_str(category));

        while !to_develop.is_empty() {
            let category = to_develop.pop().unwrap_or("");
            let base = self.graph.neighbors(category);
            for node in base {
                if !words_list.contains(&String::from(node)) {
                    words_list.push(String::from(node));
                    to_develop.push(node);
                }
            }
        }
        words_list
    }

    pub fn is_word_in_category(&self, word: String, category: String) -> bool {
        self.get_words_from_category(category).contains(&word)
    }
}