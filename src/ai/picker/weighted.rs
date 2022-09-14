use bevy::prelude::Query;
use big_brain::{choices::Choice, prelude::Picker, scorers::Score};
use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng};

#[derive(Debug, Clone, Default)]
pub struct WeightedPicker;

impl Picker for WeightedPicker {
    fn pick<'a>(&self, choices: &'a [Choice], scores: &Query<&Score>) -> Option<&'a Choice> {
        if choices.is_empty() {
            return None;
        }

        let mut rng = thread_rng();

        let weights = choices
            .iter()
            .map(|choice| choice.calculate(scores))
            .collect::<Vec<_>>();

        let dist = WeightedIndex::new(&weights).unwrap();

        Some(&choices[dist.sample(&mut rng)])
    }
}
