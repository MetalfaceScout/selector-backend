mod selection;
mod stats;
mod db;
mod output;

use core::panic;
use clap::Parser;
use output::output_game_as_json;
use serde::Serialize;

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

#[derive(Clone, Serialize)]
struct PlayerSlot {
    position: u64,
    position_pretty: String, 
    player_id: i64,
    smvp: f64,
    player_name: String
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
        players.push(stats::retrieve_stats(player, args.mvp_calculation_mode.clone()));
    }

    let mut game = create_game(args.game_type, args.team_count);

    match args.algorithm {
        Algorithm::Advanced_Selection => {
            game = selection::calculate_advanced(game, players, &args.modifier_position, &args.modifier_team);
        }
        Algorithm::AI_Selection => {
            unimplemented!("AI assisted selection is not here yet.");
        }
        Algorithm::Random_Random => {
            game = selection::random_random(game, players, &args.modifier_position, &args.modifier_team);
        }
        Algorithm::Simple_Selection => {
            unimplemented!("Simple selection is not ready yet.");
        }
    }

    output_game_as_json(game);

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
                            position: *player,
                            position_pretty: String::from(""),
                            player_id: -1,
                            smvp: -1.0,
                            player_name : "Unfilled slot! Report to Metalface!".to_string()
                        }
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