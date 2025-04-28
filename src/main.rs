use std::{collections::HashMap, env::{self}, u64};

use clap::Parser;
use dotenvy::dotenv;
use postgres::Client;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;

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

struct PlayerStats {
    player_id: u64,
    commander_stats: PostionStats,
    heavy_stats: PostionStats,
    scout_stats: PostionStats,
    ammo_stats: PostionStats,
    medic_stats: PostionStats,
    total_games_played: u64
}

#[derive(Clone)]
struct PostionStats {
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
    let mut all_games_played = 0u64;

    let mut stats = HashMap::new();

    for position in 0u64..5 {

        let position_string = set_position_from_index(position);

        let query = format!("SELECT mvp_points, hit_diff FROM scorecards WHERE id = {} AND position = '{}'", player_id, position_string);
        let results = db_client.query(&query, &[]).expect("Failed to retrieve player stats");

        let mut all_mvps: Vec<f64> = Vec::new();
        let mut all_hit_diffs: Vec<f64> = Vec::new();
        let mut game_count: u64 = 0;

        for row in &results {
            all_mvps.push(row.get(0));
            all_hit_diffs.push(row.get(1));
            game_count += 1;
        }

        let mvp_stats = calculate_stats(all_mvps, mvp_calc_method.clone());
        let hit_diff_stats = calculate_stats(all_hit_diffs, mvp_calc_method.clone());

        let position_stats = PostionStats {
            hit_diff: hit_diff_stats,
            mvp: mvp_stats,
            games_played: game_count
        };

        stats.insert(position_string, position_stats);
        all_games_played += game_count;
    }

    let final_stats = PlayerStats {
        player_id: player_id,
        commander_stats: stats.get("Commander").unwrap().clone(),
        heavy_stats: stats.get("Heavy Weapons").unwrap().clone(),
        scout_stats: stats.get("Scout").unwrap().clone(),
        ammo_stats: stats.get("Ammo Carrier").unwrap().clone(),
        medic_stats: stats.get("Medic").unwrap().clone(),
        total_games_played: all_games_played,
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
            return find_median(&stats).unwrap_or(7.0);
        }
        MVPCalculationMode::Mean => {
            return find_mean(&stats).unwrap_or(7.0);
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
            expected_player_count = 12;
        }
        
        GameType::Sm5_10_Player => {
            expected_player_count = 10;
        }

        GameType::Sm5_14_Player => {
            expected_player_count = 14;
        }

        GameType::Sm5_QueenBee => {
            expected_player_count = 8;
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

struct PlayerSlot {
    position : u64,
    player_id : i64
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
                            position: *player, player_id: -1 }
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

fn calculate_advanced(game : Vec<Vec<PlayerSlot>>, players : Vec<PlayerStats>, modifiers_position : &Option<Vec<u64>>, modifiers_team : &Option<Vec<u64>>) {
    modifiers_position;
    modifiers_team;

    // Choose a random position from a random team
    // Assign a player to that
    // Now look for the closest matching player from playerstats
    // Assign that player to the other team

    for player in players {
        println!("Player {} with commander mvp {} and hit diff {}", player.player_id, player.commander_stats.mvp, player.commander_stats.hit_diff);
    }

}