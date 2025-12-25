#[derive(Debug)]
pub struct ChessPosition {
    rank: i32,
    file: i32,
}

#[derive(Debug)]
pub struct Queen {
    pos: ChessPosition,
}

impl ChessPosition {
    pub fn new(rank: i32, file: i32) -> Option<Self> {
        let range = 0..8;
        if range.contains(&rank) && range.contains(&file) {
            Some(Self { rank, file })
        } else {
            None
        }
    }
}

impl Queen {
    pub fn new(position: ChessPosition) -> Self {
        Self { pos: position }
    }

    pub fn can_attack(&self, other: &Queen) -> bool {
        let delta_x = (self.pos.file - other.pos.file).abs();
        let delta_y = (self.pos.rank - other.pos.rank).abs();
        delta_x == 0 || delta_y == 0 || delta_x == delta_y

    }
}
