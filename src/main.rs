
mod translate;

use std::{cmp::Reverse, collections::{BTreeSet, BinaryHeap, HashMap, HashSet}, num::NonZeroU8, path::PathBuf};
use indexmap::IndexSet;
use petgraph::{csr::Csr, visit::EdgeRef, Directed};
use rusqlite::{Connection, OpenFlags};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short, help = "Path to the working database file")]
    file: PathBuf
}

fn dijkstra(graph: &Csr<(), u8, Directed, u32>) -> Vec<u32> {
    let mut seen = HashSet::new();
    let mut dist = vec![u32::MAX; graph.node_count()];
    let mut pred = vec![None; graph.node_count()];

    let mut q = BinaryHeap::new();
    dist[0] = 0;
    q.push(Reverse((0, 0)));
    for v in 1..graph.node_count() as u32 {
        q.push(Reverse((u32::MAX, v)));
    }

    while let Some(Reverse((_, u))) = q.pop() {
        if !seen.insert(u) { continue; }

        for e in graph.edges(u) {
            let v = e.target();
            let alt = dist[u as usize].checked_add(*e.weight() as u32);
            if let Some(alt) = alt && alt < dist[v as usize] {
                pred[v as usize] = Some(u);
                dist[v as usize] = alt;
                q.push(Reverse((alt, v)));
            }
        }
    }

    assert!(dist.into_iter().all(|d| d != u32::MAX));
    assert!(pred[1..].iter().all(|p| p.is_some()));
    pred[0] = Some(0);

    pred.into_iter().collect::<Option<_>>().unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut db = Connection::open_with_flags(
        args.file,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX
    )?;
    db.pragma_update(None, "foreign_keys", true)?;

    db.execute("
        CREATE TABLE IF NOT EXISTS dialogueTl (
            scriptid INTEGER,
            address INTEGER,
            tl_body TEXT NOT NULL,
            tl_variant_body TEXT,
            PRIMARY KEY (scriptid, address),
            FOREIGN KEY (scriptid, address) REFERENCES dialogue)
        WITHOUT ROWID, STRICT
    ", ())?;

    let vertices = {
        let mut stmt = db.prepare("
            SELECT tScriptid, tThread FROM graph
            UNION SELECT hScriptid, hThread FROM graph
            UNION SELECT scriptid, thread FROM dialogue")?;
        stmt.query_map((), |row| <(u16, String)>::try_from(row))?.collect::<Result<IndexSet<_>, _>>()?
    };

    let rem = {
        let mut stmt = db.prepare("
            SELECT scriptid, thread, (COUNT(body) - COUNT(tl_body)) as rem
            FROM dialogue LEFT NATURAL JOIN dialogueTl
            GROUP BY scriptid, thread
            HAVING rem != 0")?;
        stmt.query_map((), |row| {
            let (scriptid, thread, rem) = row.try_into()?;
            Ok(((scriptid, thread), rem))
        })?.collect::<Result<HashMap<(u16, String), NonZeroU8>, _>>()?
    };

    let mut graph = Csr::<(), u8, Directed, u32>::with_nodes(vertices.len() + 1);

    {
        let mut stmt = db.prepare("
            SELECT tScriptid, tThread, hScriptid, hThread, count(body)
            FROM graph LEFT JOIN dialogue ON (hScriptid, hThread) = (scriptid, thread)
            GROUP BY tScriptid, tThread, hScriptid, hThread")?;
        let mut rows = stmt.query(())?;
        while let Some(row) = rows.next()? {
            let (t_scriptid, t_thread, h_scriptid, h_thread, weight): (u16, String, u16, String, u8) = row.try_into()?;
            let t_idx = vertices.get_index_of(&(t_scriptid, t_thread)).unwrap();
            let h_idx = vertices.get_index_of(&(h_scriptid, h_thread)).unwrap();
            let added = graph.add_edge(t_idx as u32 + 1, h_idx as u32 + 1, weight);
            assert!(added);
        }
    }

    {
        let mut stmt = db.prepare("
            WITH tops(scriptid, thread) AS (
                SELECT tScriptid, tThread FROM graph
                UNION SELECT scriptid, thread FROM dialogue
                EXCEPT SELECT hScriptid, hThread from graph)
            SELECT scriptid, thread, count(body)
            FROM tops LEFT NATURAL JOIN dialogue
            GROUP BY scriptid, thread")?;
        let mut rows = stmt.query(())?;
        
        while let Some(row) = rows.next()? {
            let (h_scriptid, h_thread, weight) = row.try_into()?;
            let h_idx = vertices.get_index_of(&(h_scriptid, h_thread)).unwrap();
            let added = graph.add_edge(0, h_idx as u32 + 1, weight);
            assert!(added);
        }
    }

    let pred = dijkstra(&graph);
    let nodes = (0..graph.node_count() as u32).collect::<BTreeSet<_>>();

    let cli = reqwest::Client::new();
    
    let mut n = 0;
    for &(mut leaf) in nodes.difference(&pred.iter().copied().collect()) {
        n += 1;

        let mut path = vec![leaf];
        while pred[leaf as usize] != 0 {
            path.push(pred[leaf as usize]);
            leaf = pred[leaf as usize];
        }
        let series = path.into_iter().rev().map(|v| vertices.get_index(v as usize - 1).unwrap()).collect::<Vec<_>>();

        if series.iter().all(|&v| rem.get(v).is_none()) {
            // we've done all of these already
            continue;
        }

        for &&(scriptid, ref thread) in &series {
            eprint!("-> {scriptid}:{thread} ");
        }

        eprintln!("\n");

        if let Err(e) = translate::run(&cli, &mut db, series).await {
            eprintln!("SERIES FAILED: {e:?}");
        }

        eprintln!();
    }
    eprintln!("{n}");

    Ok(())
}
