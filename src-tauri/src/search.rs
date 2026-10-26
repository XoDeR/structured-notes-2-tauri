use std::collections::HashMap;

use rusqlite::{Connection, params_from_iter};

use crate::error::Result;
use crate::models::{SearchMode, SearchQuery, SearchResult};

const LIMIT: usize = 200;

/// Finds topics by text, tag and/or favorite flag; a topic shows up once per node it lives in.
pub fn run(conn: &Connection, query: &SearchQuery) -> Result<Vec<SearchResult>> {
    let mut clauses: Vec<String> = Vec::new();
    let mut args: Vec<String> = Vec::new();

    if query.fav {
        clauses.push("t.is_favorite = 1".into());
    }

    let tag = query.tag.trim();
    if !tag.is_empty() {
        clauses
            .push("EXISTS (SELECT 1 FROM tags g WHERE g.topic_id = t.id AND g.value = ?)".into());
        args.push(tag.to_owned());
    }

    let text = query.q.trim();
    let terms: Vec<&str> = match query.mode {
        _ if text.is_empty() => Vec::new(),
        SearchMode::Phrase => vec![text],
        SearchMode::All => text.split_whitespace().collect(),
    };
    for term in terms {
        clauses.push(if query.title_only {
            "ci_contains(t.title, ?)".into()
        } else {
            "(ci_contains(t.title, ?) OR ci_contains(t.content, ?))".into()
        });
        args.push(term.to_owned());
        if !query.title_only {
            args.push(term.to_owned());
        }
    }

    let filter = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };
    let sql = format!(
        "SELECT t.id, t.title FROM topics t {filter} ORDER BY t.title COLLATE NOCASE LIMIT {LIMIT}"
    );

    let topics: Vec<(String, String)> = conn
        .prepare(&sql)?
        .query_map(params_from_iter(&args), |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;

    let tree = Tree::load(conn)?;
    let mut places = conn.prepare("SELECT node_id FROM node_topic WHERE topic_id = ?1")?;

    let mut results = Vec::new();
    for (topic_id, title) in topics {
        let nodes: Vec<String> = places
            .query_map([&topic_id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for node_id in nodes {
            results.push(SearchResult {
                path: tree.path(&node_id),
                topic_id: topic_id.clone(),
                node_id,
                title: title.clone(),
            });
        }
    }
    Ok(results)
}

struct Tree(HashMap<String, (String, Option<String>)>);

impl Tree {
    fn load(conn: &Connection) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT id, title, parent_id FROM nodes")?;
        let map = stmt
            .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(Self(map))
    }

    /// `Notes › Work › Ideas`
    fn path(&self, id: &str) -> String {
        let mut parts = Vec::new();
        let mut current = Some(id);
        while let Some((title, parent)) = current.and_then(|id| self.0.get(id)) {
            parts.push(title.as_str());
            current = parent.as_deref();
        }
        parts.reverse();
        parts.join(" › ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use crate::models::TopicInput;
    use crate::{nodes, topics};

    fn add(conn: &Connection, node: &str, title: &str, content: &str, tags: &[&str]) -> String {
        topics::create(
            conn,
            node,
            &TopicInput {
                title: title.into(),
                content: content.into(),
                tags: tags.iter().map(|t| t.to_string()).collect(),
            },
        )
        .unwrap()
    }

    fn query(q: &str) -> SearchQuery {
        SearchQuery {
            q: q.into(),
            ..Default::default()
        }
    }

    #[test]
    fn matches_case_insensitively_beyond_ascii_and_reports_the_path() {
        let conn = test_conn();
        let root = nodes::list(&conn).unwrap()[0].id.clone();
        let work = nodes::create(&conn, &root, "Work").unwrap();
        add(&conn, &work, "Привіт світ", "<p>тіло</p>", &[]);

        let found = run(&conn, &query("ПРИВІТ")).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, "Notes › Work");
    }

    #[test]
    fn phrase_vs_all_words_and_title_only() {
        let conn = test_conn();
        let root = nodes::list(&conn).unwrap()[0].id.clone();
        add(&conn, &root, "Alpha", "<p>red green</p>", &[]);

        assert!(run(&conn, &query("green red")).unwrap().is_empty());
        let all = SearchQuery {
            mode: SearchMode::All,
            ..query("green red")
        };
        assert_eq!(run(&conn, &all).unwrap().len(), 1);
        let titles = SearchQuery {
            title_only: true,
            ..query("green")
        };
        assert!(run(&conn, &titles).unwrap().is_empty());
    }

    #[test]
    fn filters_by_tag_and_favorite() {
        let conn = test_conn();
        let root = nodes::list(&conn).unwrap()[0].id.clone();
        let a = add(&conn, &root, "A", "", &["x"]);
        add(&conn, &root, "B", "", &[]);
        topics::toggle_favorite(&conn, &a).unwrap();

        let by_tag = SearchQuery {
            tag: "x".into(),
            ..Default::default()
        };
        assert_eq!(run(&conn, &by_tag).unwrap().len(), 1);
        let favs = SearchQuery {
            fav: true,
            ..Default::default()
        };
        assert_eq!(run(&conn, &favs).unwrap()[0].topic_id, a);
    }
}
