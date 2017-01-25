use petgraph::graphmap::DiGraphMap;
use std::fs::File;
use std::io::prelude::*;
use std::mem;

pub struct WordsManager {
    pub graph: DiGraphMap<&'static str, i64>,
    pub path: String,
}

fn string_to_static_str(s: String) -> &'static str {
    unsafe {
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

/**
 * The WordsManager manages words. It constructs a Graph to classify words based on the content of
 * wordsclassification. In this file, words are stored like: word:catgory (for example blue:color)
 */
impl WordsManager {
    /**
     * Create the WordsManager
     * @param: word_file_path the path of the wordsclassification file
     * @return a WordsManager
     */
    pub fn new(word_file_path: String) -> WordsManager {
        let mut file = File::open(word_file_path.clone())
            .ok()
            .expect("Can't find modules for this type");
        let mut to_parse = String::new();
        file.read_to_string(&mut to_parse)
            .ok()
            .expect("failed to read module list");
        WordsManager {
            graph: WordsManager::build_graph(to_parse),
            path: word_file_path,
        }
    }

    /**
     * Construct the graph to describes words classification
     * @param to_parse the string to process
     * @return the graph
     */
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

    /**
     * @param category: the category to develop
     * @return all words in this category
     */
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

    /**
     * @param word: the word to find in category
     * @param category: the category to develop
     * @return true if the word is in the category, false else
     */
    pub fn is_word_in_category(&self, word: String, category: String) -> bool {
        self.get_words_from_category(category).contains(&word)
    }

    /**
     * Add a word to a category
     * @param word: the word to add
     * @param category
     */
    pub fn add_word_to_category(&mut self, word: String, category: String) {
        let child_node_key = string_to_static_str(word);
        let parent_node_key = string_to_static_str(category);

        self.graph.add_node(child_node_key);
        self.graph.add_node(parent_node_key);
        self.graph.add_edge(parent_node_key, child_node_key, 1);
        self.save();
    }

    /**
     * Remove a word to a category
     * @param word: the word to add
     * @param category
     */
    pub fn remove_word_from_category(&mut self, word: String, category: String) {
        let child_node_key = string_to_static_str(word);
        let parent_node_key = string_to_static_str(category);

        self.graph.remove_edge(parent_node_key, child_node_key);
        if self.graph.neighbors(parent_node_key).count() == 0 {
            self.graph.remove_node(parent_node_key);
        }
        if self.graph.neighbors(child_node_key).count() == 0 {
            self.graph.remove_node(child_node_key);
        }
        self.save();
    }

    /**
     * Save the graph in a file
     */
    pub fn save(&self) {
        if self.path != String::from("") {
            let mut buffer = File::create(&*self.path).unwrap();
            for edge in self.graph.all_edges() {
                let _ = buffer.write_fmt(format_args!("{}:{}\n", edge.1, edge.0));
            }
        }
    }
}
