pub mod filesys {
    #[derive(Debug)]
    pub enum NodeType {
        File,
        Directory,
    }

    #[derive(Debug)]
    pub struct Node {
        pub name: String,
        pub node_type: NodeType,
        pub children: Option<Vec<Node>>, // None si c'est un fichier
    }

    impl Node {
        // Constructeur pour un fichier
        pub fn new_file(name: String) -> Self {
            Node {
                name,
                node_type: NodeType::File,
                children: None,
            }
        }

        // Constructeur pour un dossier
        pub fn new_directory(name: String) -> Self {
            Node {
                name,
                node_type: NodeType::Directory,
                children: Some(Vec::new()),
            }
        }

        // Ajouter un enfant à un dossier
        pub fn add_child(&mut self, child: Node) {
            if let Some(children) = &mut self.children {
                children.push(child);
            } else {
                panic!("Cannot add a child to a file node");
            }
        }
    }
}

