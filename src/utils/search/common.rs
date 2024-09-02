use std::io::Cursor;

use anyhow::{bail, Result};
use skim::{
    prelude::{Key, SkimItemReader, SkimOptionsBuilder},
    FuzzyAlgorithm, Skim, SkimOutput,
};

pub(crate) async fn get_fuzzy_options<'a>(
    query: Option<String>,
    multi: bool,
    hint: String,
    options: String,
) -> Option<SkimOutput> {
    if options.is_empty() {
        return None;
    }

    tokio::task::spawn_blocking(move || {
        let hint = format!("({hint}) > ");
        let item_reader = SkimItemReader::default();
        let rx = item_reader.of_bufread(Cursor::new(options.into_bytes()));

        let options = SkimOptionsBuilder::default()
            .query(query.as_deref())
            .multi(multi)
            .bind(vec![
                "esc:abort",
                "enter:accept",
                "ctrl-c:abort",
                "ctrl-a:toggle-all",
                "tab:toggle",
            ])
            .algorithm(FuzzyAlgorithm::SkimV2)
            .prompt(Some(&hint))
            .build()
            .unwrap();

        Skim::run_with(&options, Some(rx))
    })
    .await
    .expect("Failed to run fuzzy search")
}

pub(crate) fn handle_final_key(out: &SkimOutput, selected_items: &[String]) -> Result<()> {
    match out.final_key {
        Key::Enter => {
            if selected_items.is_empty() {
                bail!("No item selected")
            } else {
                Ok(())
            }
        }
        Key::Ctrl('c') | Key::ESC => {
            bail!("User chose to abort current operation")
        }
        _ => {
            unreachable!();
        }
    }
}
