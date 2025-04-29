use core::panic;
use std::{collections::HashMap, env::{self}, f64::MAX, u64};

use clap::Parser;
use dotenvy::dotenv;
use postgres::Client;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use rand::{rng, seq::IndexedRandom};

/// The backend of the Team Selector by Metalface - Intended to be used by the website
#[derive(Parser, Debug)]
#[command(version, long_about=None)]
struct SelectorArgs {
    /// Select the game type to use
    #[arg(short, long, default_value_t, value_enum)]
    game_type: GameType,

    /// Select the algorithm to use
    #[arg(short, long, default_value_t, value_enum)]
    algorithm: Algorithm,

    /// Select the MVP Calculation mode to use
    #[arg(short, long, default_value_t, value_enum)]
    mvp_calculation_mode: MVPCalculationMode,

    /// Use team count, for some unknown reason
    #[arg(short, long, default_value_t = 2)]
    team_count: u8,

    /// Add a player to the selection - Takes player id
    #[arg(short, long)]
    player: Vec<u64>,

    /// Modify the teams to force a player to play a position - Enter player_id followed by the positon index
    #[arg(long, num_args=2, value_name = "INT")]
    modifier_position: Option<Vec<u64>>,

    /// Modify a player to force them to be on a certain team - Enter player_id followed by the team index
    #[arg(long, num_args=2, value_name = "INT")]
    modifier_team: Option<Vec<u64>>,

    /// If using last-n calculation mode, specify the amount of games to include
    #[arg(short, long, default_value_t = 50)]
    n_games: u64
}

#[derive(clap::ValueEnum, Default, Debug, Clone)]
#[allow(non_camel_case_types)]
enum GameType {
    #[default]
    Sm5_12_Player,
    Sm5_14_Player,
    Sm5_10_Player,
    Sm5_QueenBee,
}

#[derive(clap::ValueEnum, Default, Debug, Clone)]
#[allow(non_camel_case_types)]
enum Algorithm {
    Random_Random,
    Simple_Selection,
    #[default]
    Advanced_Selection,
    AI_Selection
}

#[derive(clap::ValueEnum, Default, Debug, Clone)]
#[allow(non_camel_case_types)]
enum MVPCalculationMode {
    #[default]
    Median,
    Mean,
    Last_N_Median,
    Last_N_Mean,
}

#[derive(Clone)]
struct PlayerStats {
    player_id: u64,
    player_name: String,
    commander_stats: PositionStats,
    heavy_stats: PositionStats,
    scout_stats: PositionStats,
    ammo_stats: PositionStats,
    medic_stats: PositionStats,
    //total_games_played: u64
}

#[derive(Clone, Copy)]
struct PositionStats {
    mvp: f64,
    hit_diff: f64,
    games_played: u64
}

struct DatabaseParameters {
    host: String,
    dbname: String,
    user: String,
    password: String
}

fn create_db_client(params : DatabaseParameters) -> postgres::Client {

    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build().unwrap();
    let tls = MakeTlsConnector::new(connector);

    let db_params = format!("host={} port=5432 dbname={} user={} password={} sslmode=require", params.host, params.dbname, params.user, params.password);

    let client = Client::connect(
        &db_params, 
        tls
    );

    return client.expect("Unable to connect to lfstats database");
}

fn retrieve_stats(player_id : u64, mvp_calc_method : MVPCalculationMode) -> PlayerStats {

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

fn set_position_from_index(i : u64) -> String {
    match i {
        0 => {return "Commander".to_string();}
        1 => {return "Heavy Weapons".to_string();}
        2 => {return "Scout".to_string();}
        3 => {return "Ammo Carrier".to_string();}
        4 => {return "Medic".to_string();}
        _ => {panic!("How did we get here??");}
    }
}

fn set_database_params() -> DatabaseParameters {

    dotenv().expect("Failed to load .env file");
    
    let host = env::var("DATABASE_HOST").unwrap();
    let dbname = env::var("DATABASE_NAME").unwrap();
    let user = env::var("DATABASE_USERNAME").unwrap();
    let password = env::var("DATABASE_PASSWORD").unwrap();

    let params = DatabaseParameters { host, dbname, user, password };
    return params;
}

fn main() {
    let args = SelectorArgs::parse();

    let mut players: Vec<PlayerStats> = Vec::new();

    let expected_player_count : u64;

    match args.game_type {
        GameType::Sm5_12_Player => {
            expected_player_count = 6 * args.team_count as u64;
        }
        
        GameType::Sm5_10_Player => {
            expected_player_count = 5 * args.team_count as u64;
        }

        GameType::Sm5_14_Player => {
            expected_player_count = 7 * args.team_count as u64;
        }

        GameType::Sm5_QueenBee => {
            expected_player_count = 4 * args.team_count as u64;
        }
    }

    if args.player.len() < expected_player_count.try_into().unwrap() {
        panic!("Not enough players were given to create teams.");
    }

    for player in args.player {
        players.push(retrieve_stats(player, args.mvp_calculation_mode.clone()));
    }

    let game = create_game(args.game_type, args.team_count);

    calculate_advanced(game, players, &args.modifier_position, &args.modifier_team);

}

#[derive(Clone)]
struct PlayerSlot {
    position : u64,
    player_id : i64,
    smvp: f64,
    player_name: String
}

fn create_game(game : GameType, team_count : u8) -> Vec<Vec<PlayerSlot>> {
    match game {
        GameType::Sm5_12_Player => {

            let postion_array = vec![0, 1, 2, 2, 3, 4];
            let mut final_vec: Vec<Vec<PlayerSlot>> = Vec::new();
            for _team in 0..team_count {

                let mut player_vec: Vec<PlayerSlot> = Vec::new();
                for player in &postion_array {
                    player_vec.push(
                        PlayerSlot { 
                            position: *player, player_id: -1, smvp: -1.0, player_name : "".to_string() }
                    );
                }
                final_vec.push(player_vec);
            }

            return final_vec;
        }
        
        _ => {
            todo!("All game types");
        }
    }
}

fn calculate_advanced(mut game : Vec<Vec<PlayerSlot>>, mut players : Vec<PlayerStats>, _modifiers_position : &Option<Vec<u64>>, _modifiers_team : &Option<Vec<u64>>) {

    // Choose a random slot from a random team
    // Assign a player to that
    // Now look for the closest matching player from playerstats
    // Assign that player to the other team

    let mut rng = rng();

    // Run once per slot on one team
    for slot_index in 0..game.clone().first().unwrap().len() {

        //let slot_index = rand::random_range(0..game[0].len());

        let player = players.choose(&mut rng).unwrap().clone();
        players.retain(|p| p.player_id != player.player_id);

        game[0][slot_index].player_id = player.player_id.try_into().unwrap();

        // The position of the retrieved slot
        let position = game[0][slot_index].position;

        // Fetch stats of that position
        let stats = retrieve_stat_block_from_position(position, player.clone());

        // Match a player for each opposing team (usually just 1)
        for team_match in 1..game.len() {
            // Match the next closest player's position stats
            let smvp = calculate_smvp(stats);
                        
            game[0][slot_index].smvp = smvp;
            game[0][slot_index].player_name = player.player_name.clone();

            // At the end of this next for loop, should have the closest player in mvp
            let mut min_diff = MAX;
            let mut diff_index: usize = 0;

            let mut other_smvp_chosen: f64 = 0.0;
            let mut other_name_chosen = String::from("");


            for (index, other_player) in players.iter().enumerate() {
                let p_stats = retrieve_stat_block_from_position(position, other_player.clone());
                let other_smvp = calculate_smvp(p_stats);
                let other_name = other_player.player_name.clone();
                let diff = (smvp - other_smvp).abs();


                if diff < min_diff {
                    min_diff = diff;
                    diff_index = index;
                    other_smvp_chosen = other_smvp;
                    other_name_chosen = other_name;
                }
            }

            let matched_player = players.get(diff_index).unwrap().clone();
            players.retain(|p| p.player_id != matched_player.player_id);

            game[team_match][slot_index].player_id = matched_player.player_id as i64;
            game[team_match][slot_index].smvp = other_smvp_chosen;
            game[team_match][slot_index].player_name = other_name_chosen;
        }
    
    }

    for (i, team) in game.iter().enumerate() {
        println!("Team {}", i);
        for slot in team {
            println!("Player ID: {}, Player Position: {}, Player SMVP: {}, Player Name: {}", slot.player_id, set_position_from_index(slot.position), slot.smvp, slot.player_name);
        }
    }

}


/// Calculate the magic number that the selector will use to rank players. The way to calculate this is up for debate
fn calculate_smvp(stats : PositionStats) -> f64 {
    return (stats.hit_diff * stats.mvp) + (stats.games_played as f64 / 100.0);
}

fn retrieve_stat_block_from_position(postion : u64, player : PlayerStats) -> PositionStats {
    match postion {
        0 => {return player.commander_stats}
        1 => {return player.heavy_stats}
        2 => {return player.scout_stats}
        3 => {return player.ammo_stats}
        4 => {return player.medic_stats}
        _ => {panic!("This is wrong.")}
    }
}