use std::{any::Any, env::{self, home_dir}, fmt::format, u64};

use clap::{builder::Str, error, Parser};
use postgres::{Client, NoTls};

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

struct Player {
    id: u64,
    stats: PlayerStats
}

struct PlayerStats {
    commander_stats: PostionStats,
    heavy_stats: PostionStats,
    scout_stats: PostionStats,
    ammo_stats: PostionStats,
    medic_stats: PostionStats,
    total_games_played: u64
}

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

    let db_params = format!("host={} port=5432 dbname={} user={} password={}", params.host, params.dbname, params.user, params.password);

    let mut client = Client::connect(
        &db_params, 
        NoTls
    );

    return client.expect("Unable to connect to lfstats database");
}

enum Positions {
    Commander,
    HeavyWeapons,
    Scout,
    Ammo,
    Medic
}

fn retrieve_stats(player_id : u64, db_client : &postgres::Client) -> PlayerStats {

    let query = format!("SELECT average_mvp FROM scorecards WHERE id = {}", player_id);
    let results = db_client.query(&query, &[]).expect("Failed to retrieve player stats");

    for position in 0u64..4 {

        let all_mvps: Vec<f64> = Vec::new();
        let all_hit_diffs: Vec<f64> = Vec::new();
        let game_count: u64 = 0;

        for row in results {
            all_mvps.push(row.get(0));
            all_hit_diffs.push(row.get(0));
            game_count += 1;
        }
    }

    let s = PlayerStats { commander_stats: (), heavy_stats: (), scout_stats: (), ammo_stats: (), medic_stats: (), total_games_played: () };
    return s;
}

fn set_database_params() -> DatabaseParameters {
    let host = env::var("DATABASE_HOST").unwrap();
    let dbname = env::var("DATABASE_NAME").unwrap();
    let user = env::var("DATABASE_USERNAME").unwrap();
    let password = env::var("DATABASE_PASSWORD").unwrap();

    let params = DatabaseParameters { host, dbname, user, password };
    return params;
}

fn main() {
    let args = SelectorArgs::parse();

    let db_params = set_database_params();

    let db_client = &create_db_client(db_params);

    let mut players: Vec<PlayerStats> = Vec::new();

    if let Some(v) = args.modifier_position {
        for m in v {
            print!("{}", m);
        }
    }

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
        players.push(retrieve_stats(player, db_client));
    }
}
