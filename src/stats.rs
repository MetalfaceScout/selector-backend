use std::collections::HashMap;

use crate::{db::{create_db_client, set_database_params}, MVPCalculationMode, PlayerStats, PositionStats};

pub fn retrieve_stats(player_id : u64, mvp_calc_method : MVPCalculationMode) -> PlayerStats {

    let params = set_database_params();
    let mut db_client = create_db_client(params);
    //let mut all_games_played = 0u64;

    let query = format!("SELECT mvp_points, hit_diff, position, player_name FROM scorecards WHERE player_id = {}", player_id);
    let results = db_client.query(&query, &[]).expect("Failed to retrieve player stats");

    let player_name;

    if results.len() == 0 {
        player_name = String::from("Unknown Player");
    } else {
        player_name = results[0].get(3);
    }

    let mut stats = HashMap::new();

    for position in 0u64..5 {

        let position_string = set_position_from_index(position);

        let mut all_mvps: Vec<f64> = Vec::new();
        let mut all_hit_diffs: Vec<f64> = Vec::new();
        let mut game_count: u64 = 0;

        for row in &results {
            let pos: String = row.get(2);
            if pos == set_position_from_index(position) {
                all_mvps.push(row.get(0));
                all_hit_diffs.push(row.get(1));
                game_count += 1;
            }
        }

        let mut mvp_stats = calculate_stats(all_mvps, mvp_calc_method.clone());

        if mvp_stats == -1.0 {
            mvp_stats = 7.0;
        }

        let mut hit_diff_stats = calculate_stats(all_hit_diffs, mvp_calc_method.clone());
        
        if hit_diff_stats == -1.0 {
            hit_diff_stats = 1.0;
        }

        let position_stats = PositionStats {
            hit_diff: hit_diff_stats,
            mvp: mvp_stats,
            games_played: game_count
        };

        stats.insert(position_string, position_stats);
        //all_games_played += game_count;
    }

    let final_stats = PlayerStats {
        player_id: player_id,
        player_name: player_name,
        commander_stats: stats.get("Commander").unwrap().clone(),
        heavy_stats: stats.get("Heavy Weapons").unwrap().clone(),
        scout_stats: stats.get("Scout").unwrap().clone(),
        ammo_stats: stats.get("Ammo Carrier").unwrap().clone(),
        medic_stats: stats.get("Medic").unwrap().clone(),
        //total_games_played: all_games_played,
    };

    return final_stats;
}

fn find_median<T>(values: &[T]) -> Option<f64>
where
    T: Copy + Into<f64> + PartialOrd,
{
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let len = sorted.len();
    if len % 2 == 0 {
        let mid_right = len / 2;
        let mid_left = mid_right - 1;
        Some((sorted[mid_left].into() + sorted[mid_right].into()) / 2.0)
    } else {
        Some(sorted[len / 2].into())
    }
}

fn find_mean<T>(values: &[T]) -> Option<f64>
where
    T: Copy + Into<f64>,
{
    if values.is_empty() {
        return None; // No mean for empty input
    }

    let sum: f64 = values.iter().map(|&x| x.into()).sum();
    Some(sum / values.len() as f64)
}

fn calculate_stats(stats : Vec<f64>, mvp_calc_method : MVPCalculationMode) -> f64 {
    match mvp_calc_method {
        MVPCalculationMode::Median => {
            return find_median(&stats).unwrap_or(-1.0);
        }
        MVPCalculationMode::Mean => {
            return find_mean(&stats).unwrap_or(-1.0);
        }
        MVPCalculationMode::Last_N_Mean => {
            todo!("N mode support is not done.")
        }
        MVPCalculationMode::Last_N_Median => {
            todo!("N mode support is not done.")
        }
    }
}

pub fn set_position_from_index(i : u64) -> String {
    match i {
        0 => {return "Commander".to_string();}
        1 => {return "Heavy Weapons".to_string();}
        2 => {return "Scout".to_string();}
        3 => {return "Ammo Carrier".to_string();}
        4 => {return "Medic".to_string();}
        _ => {panic!("How did we get here??");}
    }
}