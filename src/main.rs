use git2::Repository;
use skim::prelude::*;
use std::io::Cursor;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long)]
    query: Option<String>,
}

fn main() {
    let args = Cli::from_args();

    let options = SkimOptionsBuilder::default()
        .query(args.query.as_deref())
        .build()
        .unwrap();

    let repo = Repository::open(".").unwrap();

    let branches = repo.branches(None).unwrap();

    let input = branches
        .map(|branch| branch.unwrap().0)
        .filter_map(|branch| branch.name().ok().map(|name| name.unwrap().to_string()))
        .collect::<Vec<String>>();

    // let input = "Option 1\nOption 2\nOption 3\nOption 4\nOption 5";
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input.join("\n").into_bytes()));

    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    for item in selected_items {
        println!("You selected: {}", item.output());
    }
}
