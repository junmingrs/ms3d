use bevy::ecs::resource::Resource;

#[derive(Clone, Copy)]
pub struct Block {
    x: usize,
    y: usize,
    z: usize,
    is_bomb: bool,
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
}

impl Game {
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
        }
    }
    pub fn open(&mut self, x: usize, y: usize, z: usize) {
        let block = self.get_block_mut(x, y, z);
        if block.is_some() {
            todo!("open block");
        }
    }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        let within = self.check_boundary(x, y, z);
        if !within {
            return None;
        } else {
            Some(&self.map[z][y][x])
        }
    }
    pub fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> Option<Block> {
        let within = self.check_boundary(x, y, z);
        if !within {
            return None;
        } else {
            Some(self.map[z][y][x])
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
