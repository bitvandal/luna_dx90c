use std::fs::read;

pub struct Heightmap {
    heightmap_filename: String,
    heightmap: Vec<f32>,
    height_scale: f32,
    height_offset: f32,
    num_rows: i32,
    num_cols: i32,
}

impl Heightmap {
    pub fn new() -> Heightmap {
        Heightmap {
            heightmap_filename: "".to_string(),
            heightmap: vec![],
            height_scale: 0.0,
            height_offset: 0.0,
            num_rows: 0,
            num_cols: 0,
        }
    }

    pub fn load_raw(&mut self, m: i32, n: i32, filename: &str, height_scale: f32, height_offset: f32) {
        self.heightmap_filename = filename.to_owned();
        self.height_scale = height_scale;
        self.height_offset = height_offset;
        self.num_rows = m;
        self.num_cols = n;

        self.heightmap = Vec::new();
        self.heightmap.resize((m * n) as usize, 0.0);

        let input: Vec<u8> = read(&filename[0..filename.len() - 1]).expect("Failed to load RAW file");

        // Copy the array data into a float table format and scale
        // the heights.
        for i in 0..m {
            for j in 0..n {
                let k = (i * n + j) as usize;
                self.heightmap[k] = input[k] as f32 * height_scale + height_offset;
            }
        }

        // Filter the table to smooth it out.  We do this because 256 height
        // steps is rather course.  And now that we copied the data into a
        // float-table, we have more precision.  So we can smooth things out
        // a bit by filtering the heights.
        self.filter3x3();
    }

    pub fn at(&self, i: usize, j: usize) -> f32 {
        let k = i * self.num_cols as usize + j;
        self.heightmap[k]
    }

    pub fn num_rows(&self) -> i32 {
        self.num_rows
    }

    pub fn num_cols(&self) -> i32 {
        self.num_cols
    }

    fn filter3x3(&mut self) {
        let mut temp = Vec::new();
        temp.resize((self.num_rows() * self.num_cols()) as usize, 0.0);

        for i in 0..self.num_rows() {
            for j in 0..self.num_cols() {
                let k = (i * self.num_cols() + j) as usize;
                temp[k] = self.sample_height3x3(i, j);
            }
        }
        self.heightmap = temp;
    }

    fn sample_height3x3(&self, i: i32, j: i32) -> f32 {
        // Function computes the average height of the ij element.
        // It averages itself with its eight neighbor pixels.  Note
        // that if a pixel is missing neighbor, we just don't include it
        // in the average--that is, edge pixels don't have a neighbor pixel.
        //
        // ----------
        // | 1| 2| 3|
        // ----------
        // |4 |ij| 6|
        // ----------
        // | 7| 8| 9|
        // ----------
        let mut avg = 0.0;
        let mut num = 0.0;

        for m in i - 1..=i + 1 {
            for n in j - 1..=j + 1 {
                if self.in_bounds(m, n) {
                    avg += self.at(m as usize, n as usize);
                    num += 1.0;
                }
            }
        }

        avg / num
    }

    fn in_bounds(&self, i: i32, j: i32) -> bool {
        i >= 0 &&
        i < self.num_rows() &&
        j >= 0 &&
        j < self.num_cols()
    }
}