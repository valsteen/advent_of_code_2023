use sort_by_derive::{EnumSequence, SortBy};
use std::cmp::Ordering;
use std::error::Error;
use std::io::{stdin, BufRead};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
enum Card {
    Card2,
    Card3,
    Card4,
    Card5,
    Card6,
    Card7,
    Card8,
    Card9,
    T,
    J,
    Q,
    K,
    A,
}

impl TryFrom<char> for Card {
    type Error = String;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        Ok(match c {
            'A' => Self::A,
            'K' => Self::K,
            'Q' => Self::Q,
            'J' => Self::J,
            'T' => Self::T,
            '9' => Self::Card9,
            '8' => Self::Card8,
            '7' => Self::Card7,
            '6' => Self::Card6,
            '5' => Self::Card5,
            '4' => Self::Card4,
            '3' => Self::Card3,
            '2' => Self::Card2,
            _ => Err(format!("No such card: {c}"))?,
        })
    }
}

type Hand = [Card; 5];

#[derive(Copy, Clone, EnumSequence, SortBy, Debug)]
#[sort_by(enum_sequence())]
enum HandType {
    Card(Card),
    Pair(Card),
    TwoPair(Card, Card),
    ThreeOfAKind(Card),
    FullHouse(Card, Card),
    FourOfAKind(Card),
    FiveOfAKind(Card),
}

trait CmpHand {
    fn cmp(&self, other: &Self) -> Ordering;
}

impl CmpHand for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        match (HandType::try_from(*self), HandType::try_from(*other)) {
            (Ok(self_hand), Ok(other_hand)) => self_hand.cmp(&other_hand).then_with(|| Ord::cmp(self, other)),
            (Ok(_), _) => Ordering::Greater,
            (_, Ok(_)) => Ordering::Less,
            (Err(()), Err(())) => Ord::cmp(self, other),
        }
    }
}

impl TryFrom<Hand> for HandType {
    type Error = ();

    fn try_from(mut hand: Hand) -> Result<Self, Self::Error> {
        hand.sort_unstable();
        let mut groups = hand.into_iter().fold(Vec::<HandType>::new(), |mut groups, next_card| {
            match groups.last().copied() {
                Some(HandType::FourOfAKind(card)) if card == next_card => {
                    *groups.last_mut().unwrap() = HandType::FiveOfAKind(card);
                }
                Some(HandType::ThreeOfAKind(card)) if card == next_card => {
                    *groups.last_mut().unwrap() = HandType::FourOfAKind(card);
                }
                Some(HandType::Pair(card)) if card == next_card => {
                    *groups.last_mut().unwrap() = HandType::ThreeOfAKind(card);
                }
                Some(HandType::Card(card)) if card == next_card => {
                    *groups.last_mut().unwrap() = HandType::Pair(card);
                }
                _ => groups.push(HandType::Card(next_card)),
            };
            groups
        });
        groups.sort_unstable();
        match (groups.pop().unwrap(), groups.pop()) {
            (HandType::ThreeOfAKind(highest), Some(HandType::Pair(second))) => Ok(HandType::FullHouse(highest, second)),
            (HandType::Pair(highest), Some(HandType::Pair(second))) => Ok(HandType::TwoPair(highest, second)),
            (HandType::Card(_), _) => Err(()),
            (hand, _) => Ok(hand),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let lines = lines.map_while(Result::ok);

    let mut game = lines.into_iter().try_fold(Vec::new(), |mut result, line| {
        let (cards, bid) = line
            .split_once(' ')
            .ok_or_else(|| format!("missing separator {line}"))?;
        let hand = Hand::try_from(
            cards
                .chars()
                .map(Card::try_from)
                .collect::<Result<Vec<_>, _>>()?
                .as_slice(),
        )?;
        result.push((hand, bid.parse::<usize>()?));
        Ok::<_, Box<dyn Error>>(result)
    })?;

    game.sort_by(|hand1, hand2| CmpHand::cmp(&hand1.0, &hand2.0));

    let sum = game
        .into_iter()
        .enumerate()
        .map(|(i, (_, bid))| bid * (i + 1))
        .sum::<usize>();

    println!("{sum}");
    Ok::<_, Box<dyn Error>>(())
}
