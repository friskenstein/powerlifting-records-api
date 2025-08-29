use axum::{extract::Query, response::Json};
use crate::db::DB;
use rusqlite::params_from_iter;
use std::collections::HashSet;

// We decode the query as a flat list of key/value pairs to allow
// repeated parameters like `class=100&class=110` without deserialization errors.
type QueryPairs = Vec<(String, String)>;

pub async fn get_records(Query(pairs): Query<QueryPairs>) -> Json<Vec<serde_json::Value>> {
	let conn = DB.lock().unwrap();
	let mut sql = String::from("SELECT * FROM Records WHERE 1=1");
	let mut params_vec: Vec<String> = vec![];

	// Helper to append IN (?) placeholders and extend params
	fn add_in_clause(sql: &mut String, params_vec: &mut Vec<String>, column: &str, values: &[String]) {
		if values.is_empty() { return; }
		sql.push_str(&format!(
			" AND {column} IN ({})",
			(0..values.len()).map(|_| "?").collect::<Vec<_>>().join(", ")
		));
		params_vec.extend(values.iter().cloned());
	}

	// Accumulate filters
	let mut sexes: Vec<String> = vec![];
	let mut divs: Vec<String> = vec![];
	let mut events: Vec<String> = vec![];
	let mut equips: Vec<String> = vec![];
	let mut classes: Vec<String> = vec![];
	let mut lifts: Vec<String> = vec![];

	for (k, v) in pairs.into_iter() {
		match k.as_str() {
			"sex" => sexes.push(v),
			"div" => divs.push(v),
			"event" => events.push(v),
			"equip" => equips.push(v),
			"class" => classes.push(v),
			"lift" => lifts.push(v),
			_ => {},
		}
	}

	// Expand Raw <-> Bare/Sleeves/Wraps semantics
	let has_raw = equips.iter().any(|e| e == "Raw");
	let has_any_bsw = equips.iter().any(|e| matches!(e.as_str(), "Bare" | "Sleeves" | "Wraps"));
	if has_raw {
		for e in ["Bare", "Sleeves", "Wraps"] {
			if !equips.iter().any(|x| x == e) {
				equips.push(e.to_string());
			}
		}
	} else if has_any_bsw {
		// One or more of Bare/Sleeves/Wraps requested -> include Raw too
		equips.push("Raw".to_string());
	}

	// De-duplicate while preserving order
	let mut seen = HashSet::new();
	sexes.retain(|s| seen.insert(s.clone()));
	seen.clear();
	divs.retain(|s| seen.insert(s.clone()));
	seen.clear();
	events.retain(|s| seen.insert(s.clone()));
	seen.clear();
	equips.retain(|s| seen.insert(s.clone()));
	seen.clear();
	classes.retain(|s| seen.insert(s.clone()));
	seen.clear();
	lifts.retain(|s| seen.insert(s.clone()));

	add_in_clause(&mut sql, &mut params_vec, "sex", &sexes);
	add_in_clause(&mut sql, &mut params_vec, "div", &divs);
	add_in_clause(&mut sql, &mut params_vec, "event", &events);
	add_in_clause(&mut sql, &mut params_vec, "equip", &equips);
	add_in_clause(&mut sql, &mut params_vec, "class", &classes);
	add_in_clause(&mut sql, &mut params_vec, "lift", &lifts);

	// Apply deterministic ordering per requested preference:
	// equipment (custom), weight class (numeric with SHW last), division (alpha), event (custom), lift (custom)
	sql.push_str(
		" ORDER BY \
		CASE equip \
			WHEN 'Raw' THEN 0 \
			WHEN 'Bare' THEN 0 \
			WHEN 'Sleeves' THEN 0 \
			WHEN 'Wraps' THEN 0 \
			WHEN 'Single-ply' THEN 1 \
			WHEN 'Multi-ply' THEN 2 \
			WHEN 'Unlimited' THEN 3 \
			ELSE 4 \
		END, \
		CASE WHEN class = 'SHW' THEN 1000 ELSE CAST(class AS REAL) END, \
		div, \
		CASE event \
			WHEN 'SBD' THEN 0 \
			WHEN 'B' THEN 1 \
			WHEN 'D' THEN 2 \
			ELSE 3 \
		END, \
		CASE lift \
			WHEN 'S' THEN 0 \
			WHEN 'B' THEN 1 \
			WHEN 'D' THEN 2 \
			WHEN 'SBD' THEN 3 \
			ELSE 4 \
		END"
	);

	let mut stmt = conn.prepare(&sql).unwrap();
	let rows = stmt.query_map(params_from_iter(params_vec), |row| {
		Ok(serde_json::json!({
			"sex": row.get::<_, String>(1)?,
			"div": row.get::<_, String>(2)?,
			"event": row.get::<_, String>(3)?,
			"equip": row.get::<_, String>(4)?,
			"class": row.get::<_, String>(5)?,
			"lift": row.get::<_, String>(6)?,
			"weight": row.get::<_, f64>(7)?,
			"name": row.get::<_, Option<String>>(8)?,
			"date": row.get::<_, Option<String>>(9)?,
			"place": row.get::<_, Option<String>>(10)?,
		}))
	}).unwrap();

	Json(rows.filter_map(Result::ok).collect())
}

pub async fn get_errors() -> Json<Vec<serde_json::Value>> {
    let conn = DB.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT file, line, key, weight, name, date, place, reason FROM Errors ORDER BY file, line")
        .unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "file": row.get::<_, String>(0)?,
                "line": row.get::<_, i64>(1)?,
                "key": row.get::<_, Option<String>>(2)?,
                "weight": row.get::<_, Option<f64>>(3)?,
                "name": row.get::<_, Option<String>>(4)?,
                "date": row.get::<_, Option<String>>(5)?,
                "place": row.get::<_, Option<String>>(6)?,
                "reason": row.get::<_, String>(7)?,
            }))
        })
        .unwrap();

    Json(rows.filter_map(Result::ok).collect())
}
