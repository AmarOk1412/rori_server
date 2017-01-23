extern crate main;

#[cfg(test)]
mod tests_api {
    use main::core::words_manager::*;

    #[test]
    fn test_graph() {
        let to_parse = String::from("child1:parent
child2:parent
child3:parent
childchild:child2
childchildchild:childchild
child1:childchildchild");
        let wm = WordsManager { graph: WordsManager::build_graph(to_parse) };
        assert_eq!(wm.graph.node_count(), 6);

        let in_parent = wm.get_words_from_category(String::from("parent"));
        assert_eq!(in_parent.len(), 5);

        let in_child2 = wm.get_words_from_category(String::from("child2"));
        assert_eq!(in_child2.len(), 3); //child1, childchild, childchildchild

        let in_none = wm.get_words_from_category(String::from("none"));
        assert_eq!(in_none.len(), 0);

        assert!(wm.is_word_in_category(String::from("child1"), String::from("parent")));
        assert!(!wm.is_word_in_category(String::from("none"), String::from("parent")));
        assert!(!wm.is_word_in_category(String::from("child1"), String::from("none")));
        assert!(!wm.is_word_in_category(String::from("none2"), String::from("none1")));
    }
}
