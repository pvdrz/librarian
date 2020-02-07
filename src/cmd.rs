use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    about = "\"Each one of these souls is finite and precious. And I'm close... Close to saving them all.\""
)]
pub enum Command {
    #[structopt(about = "Store a new book into the library")]
    Store {
        #[structopt(help = "Path to the book to be stored")]
        file: String,
        #[structopt(short, long, help = "Title of the book", conflicts_with = "isbn")]
        title: Option<String>,
        #[structopt(short, long, help = "Authors of the book", conflicts_with = "isbn")]
        authors: Vec<String>,
        #[structopt(short, long, help = "Get book information from Open Library using the ISBN", conflicts_with = "title", conflicts_with = "authors")]
        isbn: Option<String>,
        #[structopt(short, long, help = "Keywords for the book")]
        keywords: Vec<String>,
    },
    #[structopt(about = "Find a book in the library")]
    Find {
        #[structopt(short, long, help = "Title of the book")]
        title: String,
        // #[structopt(short, long, help = "Authors of the book")]
        // authors: Vec<String>,
        // #[structopt(short, long, help = "Keywords for the book")]
        // keywords: Vec<String>,
    },
    // #[structopt(about = "Updates the info of a specific book")]
    // Update {
    //     #[structopt(about = "Hash of the book to be updated")]
    //     hash: String,
    //     #[structopt(short, long, help = "New title of the book")]
    //     title: String,
    //     #[structopt(short, long, help = "New list of authors of the book")]
    //     authors: Vec<String>,
    //     #[structopt(short, long, help = "New list of Keywords for the book")]
    //     keywords: Vec<String>,
    // },
    // #[structopt(about = "Extends the authors/keywords list of a book")]
    // Add {
    //     #[structopt(help = "Hash of the book to be updated")]
    //     hash: String,
    //     #[structopt(short, long, help = "Authors to be added to the book")]
    //     authors: Vec<String>,
    //     #[structopt(short, long, help = "Keywords to be added to the book")]
    //     keywords: Vec<String>,
    // },
    #[structopt(about = "Open a book")]
    Open {
        #[structopt(help = "Hash of the book to be opened")]
        hash: String,
    }
}

