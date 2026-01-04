use std::collections::HashMap;

type HandRank = (Vec<u8>, Vec<u8>);

#[derive(Debug)]
struct Card {
    rank: u8,
    suit: char,
}

impl Card {
    fn parse(source: &str) -> Self {
        let (rank_str, suit_str) = source.split_at(source.len() - 1);

        let rank = match rank_str {
            "A" => 14,
            "K" => 13,
            "Q" => 12,
            "J" => 11,
            n => n.parse().unwrap(),
        };

        Self {
            rank,
            suit: suit_str.chars().next().unwrap(),
        }
    }
}

#[derive(Debug)]
struct Hand<'a> {
    source: &'a str,
    rank: HandRank,
}

impl<'a> Hand<'a> {
    fn new(source: &'a str) -> Self {
        let mut cards: Vec<Card> = source.split_whitespace().map(Card::parse).collect();
        cards.sort_unstable_by(|a, b| b.rank.cmp(&a.rank));

        let rank = Self::evaluate(&cards);
        Self { source, rank }
    }

    fn evaluate(cards: &[Card]) -> HandRank {
        let flush = cards.iter().all(|c| c.suit == cards[0].suit);
        let straight = Self::is_straight(cards);

        // For ace-low straight (wheel), high card is 5
        let straight_high = if straight && cards[0].rank == 14 && cards[1].rank == 5 {
            5
        } else {
            cards[0].rank
        };

        match (flush, straight) {
            (true, true) => (vec![4, 2], vec![straight_high]),
            (true, false) => (vec![3, 1, 3], cards.iter().map(|c| c.rank).collect()),
            (false, true) => (vec![3, 1, 2], vec![straight_high]),
            _ => Self::partition_rank(cards),
        }
    }

    fn is_straight(cards: &[Card]) -> bool {
        let consecutive = cards.windows(2).all(|w| w[0].rank == w[1].rank + 1);
        // Check for wheel: A-5-4-3-2
        consecutive
            || (cards[0].rank == 14
                && cards[1].rank == 5
                && cards[1..].windows(2).all(|w| w[0].rank == w[1].rank + 1))
    }

    fn partition_rank(cards: &[Card]) -> HandRank {
        let mut histogram: HashMap<u8, u8> = HashMap::new();
        for card in cards {
            *histogram.entry(card.rank).or_insert(0) += 1;
        }

        // Sort by (count desc, rank desc)
        let mut ranked: Vec<_> = histogram.into_iter().map(|(r, c)| (c, r)).collect();
        ranked.sort_unstable_by(|a, b| b.cmp(a));

        let partition = ranked.iter().map(|(c, _)| *c).collect();
        let tiebreakers = ranked.iter().map(|(_, r)| *r).collect();
        (partition, tiebreakers)
    }
}

/// Given a list of poker hands, return a list of those hands which win.
pub fn winning_hands<'a>(hands: &[&'a str]) -> Vec<&'a str> {
    let parsed: Vec<Hand> = hands.iter().map(|&h| Hand::new(h)).collect();

    let max_rank = parsed.iter().map(|h| &h.rank).max().unwrap();

    parsed
        .iter()
        .filter(|h| &h.rank == max_rank)
        .map(|h| h.source)
        .collect()
}
