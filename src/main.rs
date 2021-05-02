use anyhow::{Context, Result};
use scraper::Html;

mod scrape;

const ARCHIVE: &str = "https://www.dhammatalks.org/mp3_index.html";
const ROOT: &str = "https://www.dhammatalks.org/";

fn main() -> Result<()> {
    let body = attohttpc::get(ARCHIVE)
        .send()
        .context("GET archive")?
        .text()
        .context("reading archive response body")?;

    let document = Html::parse_document(&body);
    let talks = scrape::TalkInfo::from_archive(&document).context("parsing talks")?;

    Ok(())
}
