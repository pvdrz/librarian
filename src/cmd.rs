use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    about = "\"Each one of these souls is finite and precious. And I'm close... Close to saving them all.\""
)]
pub enum Command {
    #[structopt(about = "Adds a new document into the library")]
    Add {
        #[structopt(help = "Path to the document to be stored")]
        file: String,
        #[structopt(
            short,
            long,
            help = "Get document information from Open Library using the ISBN"
        )]
        isbn: Option<String>,
    },
    #[structopt(about = "Finds a document in the library")]
    Find {
        #[structopt(help = "Pattern to search in the document information")]
        pattern: String,
    },
    #[structopt(about = "List all the documents in the library")]
    List,
    #[structopt(about = "Edits the info of a specific document using the default editor")]
    Edit {
        #[structopt(about = "Hash of the document to be updated")]
        hash: String,
    },
    #[structopt(about = "Opens a document")]
    Open {
        #[structopt(help = "Hash of the document to be opened")]
        hash: String,
    },
}
