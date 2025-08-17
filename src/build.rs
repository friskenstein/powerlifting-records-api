use crate::db::DB;
use std::fs;
use std::path::Path;
use rusqlite::params;

pub fn build_records(dir_path: &str) {
    let conn = DB.lock().unwrap();
    conn.execute("DELETE FROM Records;", []).unwrap();

    let keys = generate_keys();
    for key in &keys {
        let parts: Vec<&str> = key.split('|').collect();
        conn.execute(
            "INSERT INTO Records (sex, div, event, equip, class, lift) VALUES (?, ?, ?, ?, ?, ?);",
            params![parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]],
        ).ok(); // skip duplicates
    }

    let paths = fs::read_dir(Path::new(dir_path)).unwrap();

    for file in paths.flatten().filter(|f| f.path().extension().map(|e| e == "csv").unwrap_or(false)) {
        if let Ok(content) = fs::read_to_string(file.path()) {
            for line in content.lines() {
                if line.trim().is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts = parse_csv_line(line);
                if parts.len() < 6 {
                    continue;
                }

                let key = &parts[0];
                let weight = &parts[1];
                let name = &parts[2];
                let date = &parts[3];
                let place = &parts[4];
                let _comment = &parts[5];

                let key_parts: Vec<&str> = key.split('|').collect();
                if key_parts.len() != 6 {
                    continue;
                }

                let old = conn.query_row(
                    "SELECT weight FROM Records WHERE sex = ? AND div = ? AND event = ? AND equip = ? AND class = ? AND lift = ?",
                    params![key_parts[0], key_parts[1], key_parts[2], key_parts[3], key_parts[4], key_parts[5]],
                    |row| row.get::<_, f64>(0),
                ).ok();

                let w: f64 = weight.parse().unwrap_or(0.0);
                if let Some(old_weight) = old {
                    if old_weight >= w {
                        continue;
                    }
                }

                conn.execute(
                    "UPDATE Records SET weight = ?, name = ?, date = ?, place = ? WHERE sex = ? AND div = ? AND event = ? AND equip = ? AND class = ? AND lift = ?",
                    params![w, name.as_str(), date.as_str(), place.as_str(), key_parts[0], key_parts[1], key_parts[2], key_parts[3], key_parts[4], key_parts[5]],
                ).ok();
            }
        }
    }
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut inside_quotes = false;

    for c in line.chars() {
        match c {
            '"' => inside_quotes = !inside_quotes,
            ',' if !inside_quotes => {
                parts.push(current.clone());
                current.clear();
            },
            _ => current.push(c),
        }
    }
    parts.push(current);
    parts
}


fn generate_keys() -> Vec<String> {
    let sexes = ["M", "F"];
    let male_weight_classes = ["52", "56", "60", "67.5", "75", "82.5", "90", "100", "110", "125", "140", "SHW"];
    let female_weight_classes = ["44", "48", "52", "56", "60", "67.5", "75", "82.5", "90", "100", "110", "SHW"];
    let classes: Vec<&str> = male_weight_classes.iter().chain(female_weight_classes.iter()).copied().collect();

    let base_divisions = ["Open", "Youth", "T13-15", "T16-17", "T18-19", "J20-23", "M40-44", "M45-49", "M50-54", "M55-59", "M60-64", "M65-69", "M70-74", "M75-79", "M80+"];
    let mut divisions: Vec<String> = vec![];
    for div in &base_divisions {
        divisions.push((*div).to_string());
        divisions.push(format!("{div}-D"));
    }

    let lifts = ["S", "B", "D", "SBD"];
    let events = ["SBD", "B", "D"];
    let equipment = ["Raw", "Wraps", "Sleeves", "Bare", "Single-ply", "Multi-ply"];

    let eq_order = |eq: &str| match eq {
        "Raw" | "Bare" | "Sleeves" | "Wraps" => 0,
        "Single-ply" => 1,
        "Multi-ply" => 2,
        _ => 3,
    };

    let mut records = vec![];

    for sex in &sexes {
        let weight_classes = if *sex == "M" { &male_weight_classes } else { &female_weight_classes };

        for div in &divisions {
            for event in &events {
                for lift in &lifts {
                    let valid_lift = 
                        (*lift == "B" && *event == "B") ||
                        (*lift == "D" && *event == "D") ||
                        (*event == "SBD");

                    if !valid_lift { continue; }

                    for eq in &equipment {
                        let invalid_bench_dead_eq = (*lift == "B" || *lift == "D") && (*eq == "Bare" || *eq == "Sleeves" || *eq == "Wraps");
                        let invalid_squat_total_eq = (*lift == "S" || *lift == "SBD") && (*eq == "Raw");

                        if invalid_bench_dead_eq || invalid_squat_total_eq {
                            continue;
                        }

                        for class in weight_classes {
                            records.push(format!("{sex}|{div}|{event}|{eq}|{class}|{lift}"));
                        }
                    }
                }
            }
        }
    }

    // Sort to mimic TS behavior
    // functionally unnecessary here but considering future custom in-memory data structure
    records.sort_by(|a, b| {
        let a_parts: Vec<&str> = a.split('|').collect();
        let b_parts: Vec<&str> = b.split('|').collect();

        let eq_cmp = eq_order(a_parts[3]).cmp(&eq_order(b_parts[3]));
        if eq_cmp != std::cmp::Ordering::Equal {
            return eq_cmp;
        }

        let class_idx = |c: &str| classes.iter().position(|x| *x == c).unwrap_or(999);
        class_idx(a_parts[4]).cmp(&class_idx(b_parts[4]))
    });

    records
}
