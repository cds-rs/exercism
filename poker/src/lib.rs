type HandRank = (u8, Vec<u8>);

#[derive(Debug)]
struct Hand<'a> {
    source: &'a str,
    rank: HandRank,
}

impl<'a> Hand<'a> {
    fn parse_rank(s: &str) -> u8 {
        match s {
            "A" => 14, "K" => 13, "Q" => 12, "J" => 11,
            n => n.parse().unwrap(),
        }
    }

    fn new(source: &'a str) -> Self {
        let mut ranks: Vec<u8> = source
            .split_whitespace()
            .map(|c| Self::parse_rank(&c[..c.len() - 1]))
            .collect();
        ranks.sort_unstable();

        // Wheel: A-2-3-4-5 becomes 1-2-3-4-5
        if ranks == [2, 3, 4, 5, 14] {
            ranks = vec![1, 2, 3, 4, 5];
        }

        let suits: Vec<char> = source
            .split_whitespace()
            .map(|c| c.chars().last().unwrap())
            .collect();

        let flush = suits.iter().all(|&s| s == suits[0]);
        let straight = ranks.windows(2).all(|w| w[1] == w[0] + 1);

        // Group by count: [(count, rank), ...] sorted desc
        let mut groups: Vec<(u8, u8)> = ranks
            .iter()
            .map(|&r| (ranks.iter().filter(|&&x| x == r).count() as u8, r))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        groups.sort_by(|a, b| b.cmp(a));

        let sig: Vec<u8> = groups.iter().map(|(c, _)| *c).collect();
        let tiebreaker: Vec<u8> = groups.iter().map(|(_, r)| *r).collect();

        let category = match (straight, flush, sig.as_slice()) {
            (true, true, _) => 8,      // straight flush
            (_, _, [4, 1]) => 7,       // four kind
            (_, _, [3, 2]) => 6,       // full house
            (false, true, _) => 5,     // flush
            (true, false, _) => 4,     // straight
            (_, _, [3, 1, 1]) => 3,    // three kind
            (_, _, [2, 2, 1]) => 2,    // two pair
            (_, _, [2, 1, 1, 1]) => 1, // pair
            _ => 0,                    // high card
        };

        Self {
            source,
            rank: (category, tiebreaker),
        }
    }
}

pub fn winning_hands<'a>(hands: &[&'a str]) -> Vec<&'a str> {
    let parsed: Vec<Hand> = hands.iter().map(|&h| Hand::new(h)).collect();
    let max_rank = parsed.iter().map(|h| &h.rank).max().unwrap();

    parsed
        .iter()
        .filter(|h| &h.rank == max_rank)
        .map(|h| h.source)
        .collect()
}
