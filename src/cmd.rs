use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    about = "\"Each one of these souls is finite and precious. And I'm close... Close to saving them all.\""
)]
pub enum Command {
    #[structopt(about = "Store a new document into the library")]
    Store {
        #[structopt(help = "Path to the document to be stored")]
        file: String,
        #[structopt(short, long, help = "Title of the document", conflicts_with = "isbn")]
        title: Option<String>,
        #[structopt(short, long, help = "Authors of the document", conflicts_with = "isbn")]
        authors: Vec<String>,
        #[structopt(
            short,
            long,
            help = "Get document information from Open Library using the ISBN",
            conflicts_with = "title",
            conflicts_with = "authors"
        )]
        isbn: Option<String>,
        #[structopt(short, long, help = "Keywords for the document")]
        keywords: Vec<String>,
    },
    #[structopt(about = "Find a document in the library")]
    Find {
        #[structopt(help = "Pattern to search in the document information")]
        pattern: String,
    },
    #[structopt(about = "List all the documents in the library")]
    List,
    // #[structopt(about = "Updates the info of a specific document")]
    // Update {
    //     #[structopt(about = "Hash of the document to be updated")]
    //     hash: String,
    //     #[structopt(short, long, help = "New title of the document")]
    //     title: String,
    //     #[structopt(short, long, help = "New list of authors of the document")]
    //     authors: Vec<String>,
    //     #[structopt(short, long, help = "New list of Keywords for the document")]
    //     keywords: Vec<String>,
    // },
    #[structopt(about = "Extends the authors/keywords list of a document")]
    Add {
        #[structopt(help = "Hash of the document to be updated")]
        hash: String,
        #[structopt(short, long, help = "Authors to be added to the document")]
        authors: Vec<String>,
        #[structopt(short, long, help = "Keywords to be added to the document")]
        keywords: Vec<String>,
    },
    #[structopt(about = "Open a document")]
    Open {
        #[structopt(help = "Hash of the document to be opened")]
        hash: String,
    },
}
