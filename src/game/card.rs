#[derive(Debug)]
enum Card {
    Rock,
    Paper,
    Scissors,
}

#[derive(Debug, PartialEq, Eq)]
enum PlayResult{
    Win,
    Lose,
    Draw
}

impl Card {
    fn play_against(&self, other: &Card) -> PlayResult {
        match (self, other) {
            (Card::Rock, Card::Scissors)
                | (Card::Scissors, Card::Paper)
                | (Card::Paper,  Card::Rock)
                => PlayResult::Win,
            (Card::Scissors, Card::Rock)
                | (Card::Paper, Card::Scissors)
                | (Card::Rock, Card::Paper)
                => PlayResult::Lose,
            _ => PlayResult::Draw
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_beats_rock() {
        let cases = [
            // Rock Case
            (Card::Rock, Card::Scissors, PlayResult::Win),
            (Card::Rock, Card::Paper, PlayResult::Lose),
            (Card::Rock, Card::Rock, PlayResult::Draw),

            // Paper case
            (Card::Paper, Card::Rock, PlayResult::Win),
            (Card::Paper, Card::Scissors, PlayResult::Lose),
            (Card::Paper, Card::Paper, PlayResult::Draw),

            
            // Scissors case
            (Card::Scissors, Card::Paper, PlayResult::Win),
            (Card::Scissors, Card::Rock, PlayResult::Lose),
            (Card::Scissors, Card::Scissors, PlayResult::Draw),
        ]; 

        for (card, opponent_card, expected_result) in cases {
            let result = card.play_against(&opponent_card);

            assert_eq!(result, expected_result,
                "{:?} against {:?} failed", card, opponent_card
            );    
        }
    }
}
