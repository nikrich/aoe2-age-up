use super::{Trigger, TriggerMode};
use crate::state::GameState;

pub fn evaluate(trigger: &Trigger, state: &GameState) -> bool {
    let checks = [
        (trigger.time_seconds, state.game_time_seconds),
        (trigger.villagers, state.villagers),
        (trigger.population_min, state.population.map(|p| p.0)),
        (trigger.food_min, state.food),
        (trigger.wood_min, state.wood),
        (trigger.gold_min, state.gold),
        (trigger.stone_min, state.stone),
    ];

    let active: Vec<bool> = checks
        .iter()
        .filter_map(|(target, actual)| match (target, actual) {
            (Some(t), Some(a)) => Some(*a >= *t),
            (Some(_), None) => Some(false),
            _ => None,
        })
        .collect();

    if active.is_empty() {
        return false;
    }

    match trigger.mode {
        TriggerMode::All => active.iter().all(|&b| b),
        TriggerMode::Any => active.iter().any(|&b| b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(food: u32, wood: u32, gold: u32, stone: u32, vils: u32, time: u32) -> GameState {
        GameState {
            food: Some(food),
            wood: Some(wood),
            gold: Some(gold),
            stone: Some(stone),
            villagers: Some(vils),
            population: Some((vils, 200)),
            game_time_seconds: Some(time),
            last_updated: None,
        }
    }

    fn trigger(mode: TriggerMode) -> Trigger {
        Trigger {
            time_seconds: None,
            villagers: None,
            population_min: None,
            food_min: None,
            wood_min: None,
            gold_min: None,
            stone_min: None,
            mode,
        }
    }

    #[test]
    fn test_empty_trigger_returns_false() {
        let t = trigger(TriggerMode::All);
        let s = state_with(0, 0, 0, 0, 0, 0);
        assert!(!evaluate(&t, &s));
    }

    #[test]
    fn test_single_villager_condition_met() {
        let mut t = trigger(TriggerMode::All);
        t.villagers = Some(10);
        let s = state_with(0, 0, 0, 0, 10, 0);
        assert!(evaluate(&t, &s));
    }

    #[test]
    fn test_single_villager_condition_not_met() {
        let mut t = trigger(TriggerMode::All);
        t.villagers = Some(10);
        let s = state_with(0, 0, 0, 0, 9, 0);
        assert!(!evaluate(&t, &s));
    }

    #[test]
    fn test_all_mode_requires_all_conditions() {
        let mut t = trigger(TriggerMode::All);
        t.villagers = Some(21);
        t.food_min = Some(500);

        let s1 = state_with(499, 0, 0, 0, 21, 0);
        assert!(!evaluate(&t, &s1));

        let s2 = state_with(500, 0, 0, 0, 21, 0);
        assert!(evaluate(&t, &s2));
    }

    #[test]
    fn test_any_mode_requires_one_condition() {
        let mut t = trigger(TriggerMode::Any);
        t.villagers = Some(21);
        t.food_min = Some(500);

        let s1 = state_with(0, 0, 0, 0, 21, 0);
        assert!(evaluate(&t, &s1));

        let s2 = state_with(500, 0, 0, 0, 0, 0);
        assert!(evaluate(&t, &s2));

        let s3 = state_with(0, 0, 0, 0, 0, 0);
        assert!(!evaluate(&t, &s3));
    }

    #[test]
    fn test_time_condition() {
        let mut t = trigger(TriggerMode::All);
        t.time_seconds = Some(720);

        let s1 = state_with(0, 0, 0, 0, 0, 720);
        assert!(evaluate(&t, &s1));

        let s2 = state_with(0, 0, 0, 0, 0, 719);
        assert!(!evaluate(&t, &s2));
    }

    #[test]
    fn test_none_game_state_field_fails_condition() {
        let mut t = trigger(TriggerMode::All);
        t.villagers = Some(10);
        let s = GameState::default();
        assert!(!evaluate(&t, &s));
    }

    #[test]
    fn test_population_min_condition() {
        let mut t = trigger(TriggerMode::All);
        t.population_min = Some(50);
        let mut s = state_with(0, 0, 0, 0, 0, 0);
        s.population = Some((50, 200));
        assert!(evaluate(&t, &s));
    }

    #[test]
    fn test_all_resource_conditions() {
        let mut t = trigger(TriggerMode::All);
        t.food_min = Some(100);
        t.wood_min = Some(200);
        t.gold_min = Some(50);
        t.stone_min = Some(25);

        let s1 = state_with(100, 200, 50, 25, 0, 0);
        assert!(evaluate(&t, &s1));

        let s2 = state_with(100, 199, 50, 25, 0, 0);
        assert!(!evaluate(&t, &s2));
    }
}
