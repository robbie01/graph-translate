#![allow(clippy::write_with_newline)]

mod characters;

use std::{collections::HashSet, fmt::{Display, Write as _}};

use anyhow::Context;
use reqwest::Client;
use serde_json::json;
use rusqlite::{Connection, DropBehavior};

use characters::{decode_jp_speaker, Character, EnSpeaker};

use crate::translate::llm::characters::ELEMENTS;

const N_CTX: usize = 1024;
const N_PREDICT: usize = 64;

#[derive(Debug)]
pub struct Translator {}

#[derive(Clone, Debug)]
struct Seen {
    speaker: Option<(String, String)>,
    jpline: String,
    enline: String
}

impl Display for Seen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<|start_header_id|>Japanese<|end_header_id|>]\n\n")?;
        if let Some((ref jpspeaker, _)) = self.speaker {
            write!(f, "[{jpspeaker}]: ")?;
        }
        write!(f, "{}<|eot_id|><|start_header_id|>English<|end_header_id|>\n\n", &self.jpline)?;
        if let Some((_, ref enspeaker)) = self.speaker {
            write!(f, "[{enspeaker}]: ")?;
        }
        write!(f, "{}<|eot_id|>", &self.enline)?;

        Ok(())
    }
}

fn build_header(seen: &[Seen], next_speaker: Option<&str>, next_line: &str) -> anyhow::Result<String> {
    let mut cs = seen.iter()
        .filter_map(|s| s.speaker.as_ref())
        .map(|(j, _)| j.as_str())
        .chain(next_speaker)
        .filter_map(|j| match decode_jp_speaker(j) {
            Ok(EnSpeaker::Str(_)) => None,
            Ok(EnSpeaker::Character(c)) => Some(Ok(c)),
            Err(e) => Some(Err(e))
        })
        .collect::<anyhow::Result<HashSet<&Character>>>()?;

    for c in characters::CHARACTERS.iter() {
        if cs.contains(c) { continue }
        let sp = if c.jpshort.is_empty() { c.jpspeaker } else { c.jpshort };
        if next_line.contains(sp) {
            cs.insert(c);
            continue
        }
        for (a, _) in c.aliases.iter() {
            if next_line.contains(a) {
                cs.insert(c);
                continue
            }
        }
    }

    let mut els = HashSet::<&'static str>::new();
    for s in seen {
        for &(el, elt) in ELEMENTS {
            if s.jpline.contains(el) {
                els.insert(elt);
            }
        }
    }
    for &(el, elt) in ELEMENTS {
        if next_line.contains(el) {
            els.insert(elt);
        }
    }

    let mut header = "<|begin_of_text|><|start_header_id|>Metadata<|end_header_id|>\n".to_owned();
    for c in cs {
        write!(header, "\n[character] {c}")?;
    }
    for e in els {
        write!(header, "\n{e}")?;
    }
    write!(header, "<|eot_id|>")?;

    Ok(header)
}

fn build_prompt(seen: &[Seen], next_speaker: Option<&str>, next_line: &str) -> anyhow::Result<String> {
    let mut prompt = build_header(seen, next_speaker, next_line)?;
    for s in seen {
        write!(prompt, "{s}")?;
    }
    prompt.push_str("<|start_header_id|>Japanese<|end_header_id|>\n\n");
    if let Some(next_speaker) = next_speaker {
        write!(prompt, "[{next_speaker}]: ")?;
    }
    write!(prompt, "{next_line}<|eot_id|><|start_header_id|>English<|end_header_id|>\n\n")?;
    //if let Some(enspeaker) = next_speaker.map(decode_jp_speaker).transpose()? {
    //    write!(prompt, "[{enspeaker}]:")?;
    //}

    // force punctuation
    //match next_line.chars().next() {
    //    Some('（') => write!(prompt, "(")?,
    //    Some('「') => write!(prompt, "\"")?,
    //    _ => ()
    //}

    Ok(prompt)
}

#[derive(Clone, Debug)]
struct MaxTokensReachedError(String);

impl Display for MaxTokensReachedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("maximum number of tokens reached: ")?;
        f.write_str(&self.0)
    }
}

impl std::error::Error for MaxTokensReachedError {}

async fn tokenize(client: &Client, content: &str) -> anyhow::Result<Vec<u32>> {
    client
        .post("http://127.0.0.1:8080/tokenize")
        .json(&json!({ "content": content }))
        .send().await?.error_for_status()?
        .json::<serde_json::Value>().await?
        .pointer("/tokens").context("no tokens")?
        .as_array().context("tokens is not array")?
        .iter().map(|n| Ok(n.as_u64().context("not number")?.try_into()?)).collect()
}

async fn get_completion(client: &Client, prompt: &[u32], speaker: &str) -> anyhow::Result<String> {
    let resp = client
        .post("http://127.0.0.1:8080/completion")
        .json(&json!({
             "prompt": prompt,
             "n_predict": N_PREDICT,
             "grammar": format!("root ::= \"{speaker}\" [^\\x00]*")
        }))
        .send().await?.error_for_status()?
        .json::<serde_json::Value>().await?;
    
    let content = resp
        .pointer("/content").context("no content")?
        .as_str().context("content is not string")?.to_owned();

    let stop_type = resp
        .pointer("/stop_type").context("no stop type")?
        .as_str().context("stop type is not str")?;

    if stop_type != "eos" {
        Err(MaxTokensReachedError(content).into())
    } else {
        Ok(content)
    }
}

impl Translator {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub async fn translate(&self, cli: &Client, db: &mut Connection, series: impl IntoIterator<Item = &(u16, String)>) -> anyhow::Result<()> {
        let mut seen = Vec::new();

        let mut tx = db.transaction()?;
        tx.set_drop_behavior(DropBehavior::Commit);
        let mut stmt = tx.prepare_cached("
            SELECT address, speaker, body, variant_body, tl_body
            FROM dialogue LEFT NATURAL JOIN dialogueTl
            WHERE scriptid = ? and thread = ?")?;

        for &(scriptid, ref thread) in series {
            eprintln!("\n--------- {scriptid}:{thread} ---------");
            let mut rows = stmt.query((scriptid, thread))?;
            while let Some(row) = rows.next()? {
                let (address, mut speaker, mut line, mut line_variant, translation) = <(u32, Option<String>, String, Option<String>, Option<String>)>::try_from(row)?;

                if let Some(ref mut speaker) = speaker {
                    *speaker = speaker.replace("#Name[1]", "玻ヰ璃").replace("#Name[2]", "ハイリ");
                }
                line = line.replace("#Name[1]", "玻ヰ璃").replace("#Name[2]", "ハイリ");
                if let Some(ref mut line_variant) = line_variant {
                    *line_variant = line_variant.replace("#Name[1]", "玻ヰ璃").replace("#Name[2]", "ハイリ");
                }

                if let Some(translation) = translation {
                    seen.push(Seen {
                        speaker: speaker.map(|speaker| {
                            let decoded = decode_jp_speaker(&speaker)?.to_string();
                            Ok::<_, anyhow::Error>((speaker, decoded))
                        }).transpose()?,
                        jpline: line,
                        enline: translation
                    });
                    continue;
                }

                eprintln!("address = {address:X}");
                let speaker_prefix = speaker.as_ref().map_or(Ok::<_, anyhow::Error>(String::new()),
                    |speaker| Ok(format!("[{}]: ", decode_jp_speaker(speaker)?)))?;
                
                let translation_variant = match line_variant {
                    Some(line) => {
                        // translate the variant in a vacuum
                        let mut seen = seen.clone();
                        let prompt = loop {
                            let prompt = build_prompt(&seen, speaker.as_deref(), &line)?;
                            let tokens = tokenize(cli, &prompt).await?;
                            if tokens.len() > N_CTX-N_PREDICT {
                                let md = (seen.len() / 16).max(1);
                                seen.drain(0..md);
                                continue;
                            }
                            break tokens
                        };

                        Some(get_completion(cli, &prompt, &speaker_prefix).await?
                            .strip_prefix(&speaker_prefix).unwrap().trim().to_owned())
                    },
                    None => None
                };

                let translation = {
                    let prompt = loop {
                        let prompt = build_prompt(&seen, speaker.as_deref(), &line)?;
                        let tokens = tokenize(cli, &prompt).await?;
                        if tokens.len() > N_CTX-N_PREDICT {
                            // Fairly conservative exponential reduction
                            let md = (seen.len() / 16).max(1);
                            seen.drain(0..md);
                            continue;
                        }
                        break tokens
                    };

                    get_completion(cli, &prompt, &speaker_prefix).await?
                        .strip_prefix(&speaker_prefix).unwrap().trim().to_owned()
                };

                eprintln!("{speaker_prefix}{translation}\n");
                
                if let Some(ref variant) = translation_variant {
                    eprintln!("[VARIANT] {speaker_prefix}{variant}\n");
                }

                tx.prepare_cached("
                    INSERT OR REPLACE INTO dialogueTl(scriptid, address, tl_body, tl_variant_body)
                    VALUES (?, ?, ?, ?)")?
                    .execute((scriptid, address, &translation, translation_variant))?;

                seen.push(Seen {
                    speaker: speaker.map(|speaker| {
                        let decoded = decode_jp_speaker(&speaker)?.to_string();
                        Ok::<_, anyhow::Error>((speaker, decoded))
                    }).transpose()?,
                    jpline: line,
                    enline: translation
                });
            }
            
            drop(rows);
        }
        drop(stmt);
        tx.commit()?;

        Ok(())
    }
}
