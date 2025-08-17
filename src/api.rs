use axum::{extract::Query, response::Json};
use serde::Deserialize;
use crate::db::DB;
use rusqlite::params_from_iter;

#[derive(Deserialize)]
pub struct RecordQuery {
    sex: Option<String>,
    div: Option<String>,
    event: Option<String>,
    equip: Option<String>,
    class: Option<String>,
    lift: Option<String>,
}

pub async fn get_records(Query(query): Query<RecordQuery>) -> Json<Vec<serde_json::Value>> {
    let conn = DB.lock().unwrap();
    let mut sql = String::from("SELECT * FROM Records WHERE 1=1");
    let mut params_vec = vec![];

    if let Some(v) = &query.sex   { sql.push_str(" AND sex = ?");   params_vec.push(v); }
    if let Some(v) = &query.div   { sql.push_str(" AND div = ?");   params_vec.push(v); }
    if let Some(v) = &query.event { sql.push_str(" AND event = ?"); params_vec.push(v); }
    if let Some(v) = &query.equip { sql.push_str(" AND equip = ?"); params_vec.push(v); }
    if let Some(v) = &query.class { sql.push_str(" AND class = ?"); params_vec.push(v); }
    if let Some(v) = &query.lift  { sql.push_str(" AND lift = ?");  params_vec.push(v); }

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
            "name": row.get::<_, String>(8)?,
            "date": row.get::<_, String>(9)?,
            "place": row.get::<_, String>(10)?,
        }))
    }).unwrap();

    Json(rows.filter_map(Result::ok).collect())
}
