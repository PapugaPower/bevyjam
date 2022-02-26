use bevy::prelude::*;

use crate::AppState;

pub struct GameTimer(pub Timer);

pub fn tick_game_timer(
    mut timer: ResMut<GameTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
}

pub fn check_game_over(
    timer: Res<GameTimer>,
    mut state: ResMut<State<AppState>>,
) {
    if timer.0.finished() {
        state.push(AppState::GameOverLose).unwrap();
    }
}
