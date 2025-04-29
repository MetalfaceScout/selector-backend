use crate::{stats::set_position_from_index, PlayerSlot};

pub fn output_game_as_json(mut game: Vec<Vec<PlayerSlot>>) {

    for team_index in 0..game.len() {
        for player_index in 0..game[0].len() {
            game[team_index][player_index].position_pretty = set_position_from_index(game[team_index][player_index].position);
        }
    }

    print!("{}",serde_json::to_string_pretty(&game).unwrap());
}