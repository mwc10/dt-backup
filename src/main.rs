use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use scraper::Html;
use std::fs::File;

mod feed;
mod scrape;
mod splash;

const TALKS_ARCHIVE: &str = "https://www.dhammatalks.org/mp3_index.html";
const TALKS_ROOT: &str = "https://www.dhammatalks.org";

const FEED_ROOT: &str = "http://mwc10.github.io/dt-backup/";
const FEED_XML: &str = "dhammatalks-evening.xml";
const FEED_SPLASH: &str = "index.html";
const FEED_ASSETS: &str = "assets/";
const FEED_ART: &str = "dt_art.jpeg";
const FEED_CSS: &str = "main.css";

fn print_usage() {
    println!("{} - <output directory>", env!("CARGO_BIN_NAME"));
}

fn main() -> Result<()> {
    let outdir = match get_args().context("parsing arguments")? {
        Some(a) => a,
        None => return Ok(()),
    };

    eprintln!("GETting archive");
    let body = attohttpc::get(TALKS_ARCHIVE)
        .send()
        .context("GET archive")?
        .text()
        .context("reading archive response body")?;

    eprintln!("parsing archive HTML");
    let document = Html::parse_document(&body);
    let talks = scrape::TalkInfo::from_archive(&document).context("parsing talks")?;

    eprintln!("creating output feed and index page");
    fs::create_dir_all(&outdir).context("creating output directory")?;

    let feed_file = File::create(outdir.join(FEED_XML)).context("creating output file for feed")?;
    feed::generate_feed(&talks, feed_file).context("creating feed")?;

    let splash_file =
        File::create(outdir.join(FEED_SPLASH)).context("creating output file for splash page")?;
    splash::create_index(&talks, splash_file)?;

    // copy static assets over
    let assetdir = Path::new(FEED_ASSETS);
    fs::copy(assetdir.join(FEED_ART), outdir.join(FEED_ART)).context("copying artwork")?;
    fs::copy(assetdir.join(FEED_CSS), outdir.join(FEED_CSS)).context("copying css")?;

    Ok(())
}

fn get_args() -> Result<Option<PathBuf>> {
    let mut args = std::env::args_os().skip(1);

    let outdir = match args.next() {
        Some(s) if s == "-h" || s == "--help" => {
            print_usage();
            return Ok(None);
        }
        Some(f) => PathBuf::from(f),
        None => bail!("missing output directory path"),
    };

    Ok(Some(outdir))
}
