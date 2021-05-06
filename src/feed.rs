use crate::scrape::{Talk, TalkInfo};
use anyhow::{Context, Result};
use rss::{
    extension::itunes::{
        self, ITunesCategory, ITunesChannelExtension, ITunesChannelExtensionBuilder,
    },
    Category, ChannelBuilder, Enclosure, Guid, Item,
};
use std::{collections::HashMap, fmt::Write as _, io::Write};

const TITLE: &str = "Dhammatalks.org Evening Talks";
const LINK: &str = "http://dhammatalks.org";
const LANG: &str = "en-us";
const WEBMASTER: &str = "dhammatalks.feedback@gmail.com";
const CATEGORY: &str = "Society/Religion and Spirituality/Buddhism";

pub fn generate_feed(archive: &TalkInfo, feed: impl Write) -> Result<()> {
    let TalkInfo { description, talks } = archive;

    let categories = vec![Category {
        name: CATEGORY.to_string(),
        domain: Some("https://dmoz-odp.org".to_string()),
    }];

    let items = talks_into_items(talks);

    let namespaces = {
        let mut m = HashMap::new();
        m.insert("itunes".to_string(), itunes::NAMESPACE.to_string());
        m
    };
    let itunes = itunes_ext();

    let channel = ChannelBuilder::default()
        .title(TITLE)
        .link(LINK)
        .description(*description)
        .language(LANG.to_string())
        .webmaster(WEBMASTER.to_string())
        .categories(categories)
        .items(items)
        .pub_date(chrono::Utc::now().to_rfc2822())
        .namespaces(namespaces)
        .itunes_ext(itunes)
        .build()
        .expect("infailable channel building");

    channel.write_to(feed).context("writing feed out")?;
    Ok(())
}

fn itunes_ext() -> ITunesChannelExtension {
    let category = ITunesCategory {
        text: "Religion & Spirituality".into(),
        subcategory: Some(Box::new(ITunesCategory {
            text: "Buddhism".into(),
            subcategory: None,
        })),
    };

    ITunesChannelExtensionBuilder::default()
        .author("Thanissaro Bhikkhu".to_string())
        .image(format!("{}{}", crate::FEED_ROOT, crate::FEED_ART))
        .categories(vec![category])
        .build()
        .expect("infallible")
}

fn talks_into_items(talks: &[Talk]) -> Vec<Item> {
    talks
        .into_iter()
        .map(|talk| {
            let url = format!("{}{}", crate::TALKS_ROOT, talk.mp3);
            let description = {
                let mut s = format!("A talk by Thanissaro Bhikkhu entitled \"{}\"", talk.title);
                if let Some(t) = talk.transcript {
                    writeln!(
                        &mut s,
                        "\n\nTranscript available at: {}{}",
                        crate::TALKS_ROOT,
                        t
                    )
                    .unwrap();
                }

                Some(s)
            };
            let enclosure = Some(Enclosure {
                url: url.clone(),
                mime_type: "audio/mpeg".to_string(),
                ..Default::default()
            });
            let guid = Some(Guid {
                value: url.clone(),
                ..Default::default()
            });

            Item {
                title: Some(talk.title.to_string()),
                link: Some(url),
                pub_date: Some(talk.date.to_rfc2822()),
                description,
                enclosure,
                guid,

                ..Default::default()
            }
        })
        .collect()
}
