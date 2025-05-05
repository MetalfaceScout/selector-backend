use std::f64::MAX;

use rand::{random_bool, rng, seq::IndexedRandom};

use crate::{PlayerSlot, PlayerStats, PositionStats};

pub fn random_random(mut game : Vec<Vec<PlayerSlot>>, mut players : Vec<PlayerStats>, _modifiers_position : &Option<Vec<u64>>, _modifiers_team : &Option<Vec<u64>>) -> Vec<Vec<PlayerSlot>> {

    let mut rng = rng();

    for (team_index, team) in game.clone().iter().enumerate() {
        for (slot_index, slot) in team.clone().iter().enumerate() {
            let player = players.choose(&mut rng).unwrap().clone();
            players.retain(|p| p.player_id != player.player_id);

            let player_position_stats = retrieve_stat_block_from_position(slot.position, player.clone());

            map_player_to_slot(player, player_position_stats, &mut game[team_index][slot_index]);

        }
    }

    return game;

}

pub fn calculate_advanced(mut game : Vec<Vec<PlayerSlot>>, mut players : Vec<PlayerStats>, modifiers_position : &Option<Vec<u64>>, _modifiers_team : &Option<Vec<u64>>) -> Vec<Vec<PlayerSlot>> { 

    if let Some(modifiers) = modifiers_position {
        (game, players) = assign_position_modifier_to_slot(game, players, modifiers);
    }

    // Choose a random slot from a random team
    // Assign a player to that
    // Now look for the closest matching player from playerstats
    // Assign that player to the other team

    let mut rng = rng();

    // Run once per slot on one team
    for slot_index in 0..game.clone().first().unwrap().len() {

        let player;
        let position = game[0][slot_index].position;
        let smvp;
                        
        let mut modifier_used = false;

        // Skip assigning a player if one is already assigned from modifiers

        if game[0][slot_index].player_id != -1 {
            // Find that player - grab their stats
            smvp = game[0][slot_index].smvp;
            modifier_used = true;
        } else {
            // Unassigned slot, randomly choose a player from the pool
            player = players.choose(&mut rng).unwrap().clone();
            players.retain(|p| p.player_id != player.player_id);
            game[0][slot_index].player_id = player.player_id.try_into().unwrap();

            let stats = retrieve_stat_block_from_position(position, player.clone());
            smvp = calculate_smvp(stats);

            game[0][slot_index].smvp = smvp;
            game[0][slot_index].player_name = player.player_name.clone();
        }

        // Match a player for each opposing team (usually just 1)
        for team_match in 1..game.len() {
            // Match the next closest player's position stats

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

        // Switch the slot for team 0 and 1 if the modifer was used
        if modifier_used { 
            let switch = random_bool(0.5);
            if switch {
                let temp_player_slot_0 = game[0][slot_index].clone();
                let temp_player_slot_1 = game[1][slot_index].clone();

                game[0][slot_index] = temp_player_slot_1;
                game[1][slot_index] = temp_player_slot_0;
            }
        }
    
    }

    return game;

}

fn map_player_to_slot(player : PlayerStats, pos_stats: PositionStats, slot : &mut PlayerSlot) {
    slot.player_id = player.player_id as i64;
    slot.player_name = player.player_name;
    slot.smvp = calculate_smvp(pos_stats);
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

fn assign_position_modifier_to_slot(mut game : Vec<Vec<PlayerSlot>>, mut players : Vec<PlayerStats>, modifiers_position : &Vec<u64>) -> (Vec<Vec<PlayerSlot>>, Vec<PlayerStats>) {

    for modifiers_index in (0..modifiers_position.len()).step_by(2) {
        let player_id = modifiers_position[modifiers_index];
        let position = modifiers_position[modifiers_index+1];

        let player = players.iter().find(|player| player.player_id == player_id).expect("selection: assign: Could not find passed player id in pool.").clone();
        //TODO: There has to be a better way.
        // Do I assign a player to a random team or always only the first team? 
        // Random team causes problems as the advanced selector only matches with the first team.
        let team_index = 0;

        for slot_index in 0..game[team_index].len() {
            if game[team_index][slot_index].position == position && game[team_index][slot_index].player_id == -1 {
                let player_position_stats = retrieve_stat_block_from_position(game[team_index][slot_index].position, player.clone());
                players.retain(|p| p.player_id != player.player_id);
                map_player_to_slot(player.clone(), player_position_stats, &mut game[team_index][slot_index]);
                break;
            }
        }

    }

    (game, players)
}