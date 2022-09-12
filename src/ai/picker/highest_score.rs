use bevy::prelude::Query;
use big_brain::{choices::Choice, prelude::Picker, scorers::Score};

#[derive(Debug, Clone, Default)]
pub struct HighestScorePicker {
    pub threshold: f32,
}

impl HighestScorePicker {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Picker for HighestScorePicker {
    fn pick<'a>(&self, choices: &'a [Choice], scores: &Query<&Score>) -> Option<&'a Choice> {
        let mut res: Option<&Choice> = None;
        let mut max_score: f32 = -1.0;

        for choice in choices.iter() {
            let score = choice.calculate(scores);
            if score > max_score && score > self.threshold {
                max_score = score;
                res = Some(choice);
            }
        }
        res
    }
}
