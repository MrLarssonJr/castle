use rand::random;
use crate::domain::lemonade::ports::random_provider::RandomProvider;

pub struct Random;

impl RandomProvider for Random {
	fn random(&self) -> u128 {
		random()
	}
}
