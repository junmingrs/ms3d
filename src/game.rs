use std::collections::VecDeque;

use bevy::ecs::resource::Resource;

#[derive(Clone, Copy)]
pub struct Block {
    x: usize,
    y: usize,
    z: usize,
    pub is_bomb: bool,
    pub nearby_bombs: usize,
    pub is_revealed: bool,
}

#[derive(Resource)]
pub struct Game {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    map: Vec<Vec<Vec<Block>>>,
    bombs: usize,
    pub current_layer: usize,
    pub max_layer: usize,
    is_opened: bool,
}

impl Game {
    const OFFSETS: [(i32, i32, i32); 26] = [
        (-1, -1, -1),
        (-1, -1, 0),
        (-1, -1, 1),
        (-1, 0, -1),
        (-1, 0, 0),
        (-1, 0, 1),
        (-1, 1, -1),
        (-1, 1, 0),
        (-1, 1, 1),
        (0, -1, -1),
        (0, -1, 0),
        (0, -1, 1),
        (0, 0, -1),
        (0, 0, 1),
        (0, 1, -1),
        (0, 1, 0),
        (0, 1, 1),
        (1, -1, -1),
        (1, -1, 0),
        (1, -1, 1),
        (1, 0, -1),
        (1, 0, 0),
        (1, 0, 1),
        (1, 1, -1),
        (1, 1, 0),
        (1, 1, 1),
    ];

    pub fn new(x: usize, y: usize, z: usize, bombs: usize) -> Self {
        let mut map: Vec<Vec<Vec<Block>>> = Vec::new();
        let max_layer = [x, y, z].into_iter().min().unwrap().saturating_sub(1) / 2;
        for depth in 0..z {
            let mut layer = Vec::new();
            for height in 0..y {
                let mut row = Vec::new();
                for width in 0..z {
                    row.push(Block {
                        x: width,
                        y: height,
                        z: depth,
                        is_bomb: false,
                        nearby_bombs: 0,
                        is_revealed: false,
                    });
                }
                layer.push(row);
            }
            map.push(layer);
        }

        Self {
            x,
            y,
            z,
            map,
            bombs,
            current_layer: 0,
            max_layer,
            is_opened: false,
        }
    }

    pub fn generate_bombs(&mut self, x: usize, y: usize, z: usize) {
        let mut bomb_positions: Vec<(usize, usize, usize)> = Vec::new();
        let mut remaining = self.x * self.y * self.z - 1;

        for depth in self.map.iter() {
            for height in depth {
                for row in height {
                    if self.bombs > 0 && (row.x, row.y, row.z) != (x, y, z) {
                        let is_bomb = rand::random_bool(
                            self.bombs as f64 / remaining as f64,
                        );
                        remaining -= 1;
                        if is_bomb {
                            bomb_positions.push((row.x, row.y, row.z));
                            self.bombs -= 1;
                        }
                    }
                }
            }
        }

        for (x, y, z) in bomb_positions {
            if let Some(block) = self.get_block_mut(x, y, z) {
                block.is_bomb = true;
                for (dx, dy, dz) in Self::OFFSETS {
                    if let Some(neighbour) = self.get_offset_position(x, y, z, dx, dy, dz) {
                        self.block_add_bomb(neighbour.0, neighbour.1, neighbour.2);
                    }
                }
            }
        }
    }

    fn block_add_bomb(&mut self, x: usize, y: usize, z: usize) {
        if let Some(block) = self.get_block_mut(x, y, z) {
            block.nearby_bombs += 1;
        }
    }

    pub fn open(&mut self, x: usize, y: usize, z: usize) {
        if !self.is_opened {
            self.is_opened = true;
            self.generate_bombs(x, y, z);
        }
        if let Some(block) = self.get_block_mut(x, y, z) {
            if block.is_revealed {
                // info!("already opened, {} is bomb", block.is_bomb);
                return;
            }
            // info!("opened, {} bombs, {} is bomb", block.nearby_bombs, block.is_bomb);
            block.is_revealed = true;
            let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::new();
            queue.push_back((x, y, z));

            while !queue.is_empty() {
                let (x, y, z) = queue.pop_front().unwrap();
                if let Some(block) = self.get_block_mut(x, y, z) {
                    // info!("opening!, {} bomb?", block.is_bomb);
                    block.is_revealed = true;
                    if block.nearby_bombs > 0 {
                        continue;
                    }
                    for (dx, dy, dz) in Self::OFFSETS {
                        if let Some(neighbour) = self.get_offset_position(x, y, z, dx, dy, dz)
                            && let Some(block) =
                                self.get_block(neighbour.0, neighbour.1, neighbour.2)
                            && !block.is_revealed
                            && !block.is_bomb
                        {
                            queue.push_back((neighbour.0, neighbour.1, neighbour.2));
                        }
                    }
                }
            }
        }
    }

    fn get_offset_position(
        &self,
        x: usize,
        y: usize,
        z: usize,
        dx: i32,
        dy: i32,
        dz: i32,
    ) -> Option<(usize, usize, usize)> {
        let neighbour = (x as i32 + dx, y as i32 + dy, z as i32 + dz);
        if neighbour.0 < 0 || neighbour.1 < 0 || neighbour.2 < 0 {
            return None;
        }
        Some((
            neighbour.0 as usize,
            neighbour.1 as usize,
            neighbour.2 as usize,
        ))
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        let within = self.check_boundary(x, y, z);
        if !within {
            None
        } else {
            Some(&self.map[z][y][x])
        }
    }
    pub fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Block> {
        let within = self.check_boundary(x, y, z);
        if !within {
            None
        } else {
            Some(&mut self.map[z][y][x])
        }
    }
    pub fn check_boundary(&self, x: usize, y: usize, z: usize) -> bool {
        if x >= self.x || y >= self.y || z >= self.z {
            return false;
        }
        true
    }
    // pub fn get_centre_cube(&self) -> Block {
    //     self.map[self.z / 2][self.y / 2][self.x / 2]
    // }
}
