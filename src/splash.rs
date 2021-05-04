use crate::scrape::{Talk, TalkInfo};
use anyhow::{Context, Result};
use std::io::Write;

const MAIN_BODY: &str = r##"<html>
    <head>
        <meta charset="UTF-8">
        <title>Dhammatalks.org Evening Talk Backup Podcast Feed</title>
        <link rel="stylesheet" href="main.css" /> 
    </head>
    <body>
        <h1>Dhammatalks.org Evening Talk Backup Podcast Feed</h1>
        <p>
            An alternative/backup feed of the <a href="https://www.dhammatalks.org/mp3_index_current.html">evening talks</a> 
            from <a href="https://www.dhammatalks.org/">dhammatalks.org</a>.
        </p>
        <p>
            <a href="dhammatalks-evening.xml">
                <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" aria-hidden="true" focusable="false" width="0.86em" height="1em" style="-ms-transform: rotate(360deg); -webkit-transform: rotate(360deg); transform: rotate(360deg);" preserveAspectRatio="xMidYMid meet" viewBox="0 0 1536 1792"><path d="M994 1192q0 86-17 197q-31 215-55 313q-22 90-152 90t-152-90q-24-98-55-313q-17-110-17-197q0-168 224-168t224 168zm542-424q0 240-134 434t-350 280q-8 3-15-3t-6-15q7-48 10-66q4-32 6-47q1-9 9-12q159-81 255.5-234t96.5-337q0-180-91-330.5T1070 203t-337-74q-124 7-237 61T302.5 330.5t-128 202T128 773q1 184 99 336.5T484 1341q7 3 9 12q3 21 6 45q1 9 5 32.5t6 35.5q1 9-6.5 15t-15.5 2q-148-58-261-169.5t-173.5-264T1 730q7-143 66-273.5t154.5-227T446.5 72T719 2q164-10 315.5 46.5t261 160.5t175 250.5T1536 768zm-542-32q0 93-65.5 158.5T770 960t-158.5-65.5T546 736t65.5-158.5T770 512t158.5 65.5T994 736zm288 32q0 122-53.5 228.5T1082 1174q-8 6-16 2t-10-14q-6-52-29-92q-7-10 3-20q58-54 91-127t33-155q0-111-58.5-204T938 422.5T726 386q-133 15-229 113T388 730q-10 92 23.5 176t98.5 144q10 10 3 20q-24 41-29 93q-2 9-10 13t-16-2q-95-74-148.5-183T258 757q3-131 69-244t177-181.5T745 257q144-7 268 60t196.5 187.5T1282 768z"/><rect x="0" y="0" width="1536" height="1792" fill="rgba(0, 0, 0, 0)" /></svg>
                Evening Talks Podcast Feed
            </a>
        </p>
        <img src="dt_art.jpeg" />
"##;

const HDR_RECENT_TALKS: &str = "        <h2>Recent Talks</h2>";
const HDR_RECENT_TRANS: &str = "        <h2>Recent Transcripts</h2>";

const HTML_END: &str = r#"
    </body>
</html>
"#;

pub fn create_index(info: &TalkInfo, mut out: impl Write) -> Result<()> {
    write!(&mut out, "{}", MAIN_BODY)?;
    writeln!(&mut out, "{}", HDR_RECENT_TALKS)?;
    write_lists(&mut out, &info.talks, ListKind::Recent).context("writing recent talks list")?;
    writeln!(&mut out, "{}", HDR_RECENT_TRANS)?;
    write_lists(&mut out, &info.talks, ListKind::Transcripts)
        .context("writing recent transcripts list")?;

    write!(&mut out, "{}", HTML_END)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListKind {
    Recent,
    Transcripts,
}

fn write_lists(mut out: impl Write, talks: &[Talk], kind: ListKind) -> Result<()> {
    let entries = talks
        .into_iter()
        .filter_map(|t| {
            if kind == ListKind::Recent {
                Some((t.mp3, t.date, t.title))
            } else {
                t.transcript.map(|trans| (trans, t.date, t.title))
            }
        })
        .take(5);

    writeln!(&mut out, "        <ul>")?;
    for (link, date, title) in entries {
        writeln!(
            &mut out,
            r#"            <li>{} — <a href="{}{}">{}</a></li>"#,
            date.format("%B %e, %Y").to_string(),
            crate::TALKS_ROOT,
            link,
            title
        )
        .context("writing talk entry")?;
    }
    writeln!(&mut out, "        </ul>")?;

    Ok(())
}
