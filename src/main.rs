
mod translate;

use std::{collections::{BTreeSet, HashMap, HashSet}, path::PathBuf};
use rusqlite::{Connection, OpenFlags};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short, help = "Path to the working database file")]
    file: PathBuf
}

type Thread = (u16, String);

#[expect(dead_code)]
fn spanning_tree<'a>(graph: &'a HashMap<Thread, BTreeSet<Thread>>, start: &'a Thread) -> Vec<Vec<&'a Thread>> {
    fn inner<'a>(graph: &'a HashMap<Thread, BTreeSet<Thread>>, seen: &mut HashSet<&'a Thread>, cur: &mut Vec<&'a Thread>) -> Vec<Vec<&'a Thread>> {
        let thread = cur.last().copied().unwrap();
        seen.insert(thread);

        let mut leaves = Vec::new();

        let mut leaf = true;
        if let Some(next) = graph.get(thread) {
            for next in next {
                if !seen.contains(next) {
                    leaf = false;
                    cur.push(next);
                    leaves.extend(inner(graph, seen, cur));
                    cur.pop();
                }
            }
        }
        if leaf {
            leaves.push(cur.clone())
        }
        leaves
    }

    let mut seen = HashSet::with_capacity(graph.len());
    inner(graph, &mut seen, &mut vec![start])
}

// A heuristic attempt to produce a spanning tree with many leaves (Maximum-Leaf
// Spanning Tree approximation). It performs a DFS but, at each node, visits
// unvisited neighbours in increasing order of their number of unvisited
// outgoing neighbours (so nodes with fewer extension options are explored
// first). This tends to create more leaves in the resulting spanning tree.
fn max_leaf_spanning_tree<'a>(graph: &'a HashMap<Thread, BTreeSet<Thread>>, start: &'a Thread) -> Vec<Vec<&'a Thread>> {
    fn inner<'a>(graph: &'a HashMap<Thread, BTreeSet<Thread>>, seen: &mut HashSet<&'a Thread>, cur: &mut Vec<&'a Thread>) -> Vec<Vec<&'a Thread>> {
        let thread = cur.last().copied().unwrap();
        seen.insert(thread);

        let mut leaves = Vec::new();

        let mut leaf = true;
        if let Some(next_set) = graph.get(thread) {
            // Collect unvisited neighbours
            let mut candidates: Vec<&'a Thread> = next_set.iter().filter(|n| !seen.contains(n)).collect();
            if !candidates.is_empty() {
                leaf = false;
                // Sort by increasing number of unvisited outgoing neighbours (deterministic tiebreaker by node)
                candidates.sort_by(|a, b| {
                    let a_unvisited = graph.get(a).map(|s| s.iter().filter(|x| !seen.contains(x)).count()).unwrap_or(0);
                    let b_unvisited = graph.get(b).map(|s| s.iter().filter(|x| !seen.contains(x)).count()).unwrap_or(0);
                    a_unvisited.cmp(&b_unvisited).then_with(|| a.cmp(b))
                });

                for next in candidates {
                    cur.push(next);
                    leaves.extend(inner(graph, seen, cur));
                    cur.pop();
                }
            }
        }

        if leaf {
            leaves.push(cur.clone())
        }
        leaves
    }

    let mut seen = HashSet::with_capacity(graph.len());
    inner(graph, &mut seen, &mut vec![start])
}



#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    #[expect(unused_mut)]
    let mut db = Connection::open_with_flags(
        args.file,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX
    )?;
    db.pragma_update(None, "foreign_keys", true)?;

    db.execute("
        CREATE TABLE IF NOT EXISTS dialogueTl (
            scriptid INTEGER,
            address INTEGER,
            body TEXT NOT NULL,
            variant_body TEXT,
            PRIMARY KEY (scriptid, address),
            FOREIGN KEY (scriptid, address) REFERENCES dialogue
        ) WITHOUT ROWID, STRICT
    ", ())?;

    // BTreeSet for determinism
    let mut graph = HashMap::<_, BTreeSet<_>>::new();
    let mut stmt = db.prepare("SELECT tScriptid, tThread, hScriptid, hThread FROM graph")?;
    let mut rows = stmt.query(())?;
    while let Some(row) = rows.next()? {
        let (t_scriptid, t_thread, h_scriptid, h_thread): (u16, String, u16, String) = row.try_into()?;
        graph.entry((t_scriptid, t_thread)).or_default().insert((h_scriptid, h_thread));
    }

    drop(rows);
    drop(stmt);

    let mut stmt = db.prepare("SELECT tScriptid, tThread FROM graph UNION SELECT scriptid, thread FROM dialogue EXCEPT SELECT hScriptid, hThread FROM GRAPH")?;
    let heads = stmt.query_map((), |row| <(u16, String)>::try_from(row))?.collect::<rusqlite::Result<Vec<_>>>()?;

    drop(stmt);

    for head in heads {
        let serieses = max_leaf_spanning_tree(&graph, &head);
        let avg = serieses.iter().map(|f| f.len()).sum::<usize>() as f32 / serieses.len() as f32;
        println!("{} {avg}", serieses.len());
        continue;

        #[expect(unreachable_code)]
        for series in serieses {
            translate::run(&mut db, &series).await?;
        }
    }

    Ok(())
}
