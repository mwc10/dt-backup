use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, FixedOffset, TimeZone};
use scraper::{ElementRef, Html, Selector};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TalkInfo<'a> {
    pub description: &'a str,
    pub talks: Vec<Talk<'a>>,
}

impl<'a> TalkInfo<'a> {
    pub fn from_archive(document: &'a Html) -> Result<Self> {
        let description = get_description(&document).ok_or(anyhow!("couldn't find description"))?;
        let talks = get_talks(&document).context("parsing HTML into Talks")?;

        Ok(Self { description, talks })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Talk<'a> {
    pub date: DateTime<FixedOffset>,
    pub title: &'a str,
    /// relative URL to mp3 file
    pub mp3: &'a str,
    /// relative URL to transcript PDF
    pub transcript: Option<&'a str>,
}

fn get_description(doc: &Html) -> Option<&str> {
    let content = Selector::parse("div#content").unwrap();
    let archive = Selector::parse("div.archive").unwrap();
    let desc = Selector::parse("div.full").unwrap();

    return doc
        .select(&content)
        .next()
        .and_then(|c| c.select(&archive).next())
        .and_then(|a| a.select(&desc).next())
        .and_then(|info| info.text().next());
}

fn get_talks(doc: &Html) -> Result<Vec<Talk>> {
    let audio = Selector::parse("a.audio").unwrap();
    let talks = doc.select(&audio);

    talks.into_iter().map(parse_talk).collect()
}

fn parse_talk(el: ElementRef) -> Result<Talk> {
    let mp3 = el
        .value()
        .attr("href")
        .ok_or_else(|| anyhow!("missing link to talk mp3"))?;
    let title = el
        .text()
        .nth(1)
        .ok_or_else(|| anyhow!("no talk title after date text node"))?
        .trim();
    let date = get_date(mp3).with_context(|| format!("getting date from {:?}", mp3))?;
    let transcript = el
        .next_sibling()
        .and_then(ElementRef::wrap)
        .and_then(|el| el.value().attr("href"));

    Ok(Talk {
        title,
        transcript,
        mp3,
        date,
    })
}

fn get_date(url: &str) -> Result<DateTime<FixedOffset>> {
    let mut components = url.split('/').skip(1);
    let base = components
        .next()
        .ok_or_else(|| anyhow!("missing archive directory"))?;
    let _year_dir = components
        .next()
        .ok_or_else(|| anyhow!("missing year directory"))?;
    let filename = components
        .next()
        .ok_or_else(|| anyhow!("missing mp3 file name"))?;

    if base != "Archive" {
        return Err(anyhow!("base directory is not \"Archive\""));
    }

    let year = &filename[0..2].parse().context("parsing year")? + 2000i32;
    let month = filename[2..4].parse().context("parsing month")?;
    // some of the earlier talks do not have a proper day
    // the other RSS feed listed those talks as happening on the first of the month
    let day = filename[4..6].parse().unwrap_or(1);

    const HOUR: i32 = 3600;
    let datetime = FixedOffset::west(8 * HOUR)
        .ymd(year, month, day)
        .and_hms(18, 0, 0);

    Ok(datetime)
}
