use ndarray::{s, Array1};
use serde_json::Value;

pub struct StrategyPolytope {
    pub par: Vec<usize>,
    pub idx: Vec<usize>,
}
impl StrategyPolytope {
    fn minimize(&self, mut c: Array1<f64>) -> f64 {
        for (i, &p) in self.par.iter().enumerate().rev() {
            let min = c
                .slice(s![self.idx[i]..self.idx[i + 1]])
                .fold(f64::INFINITY, |m, v| v.min(m));
            c[p] += min;
        }
        c[0]
    }
    fn maximize(&self, mut c: Array1<f64>) -> f64 {
        for (i, &p) in self.par.iter().enumerate().rev() {
            let max = c
                .slice(s![self.idx[i]..self.idx[i + 1]])
                .fold(f64::NEG_INFINITY, |m, v| v.max(m));
            c[p] += max;
        }
        c[0]
    }
}

pub struct SparseMatrix {
    cols: Vec<Vec<usize>>,
    data: Vec<Vec<f64>>,
}
impl SparseMatrix {
    fn new(n_row: usize, row: &Vec<usize>, col: &Vec<usize>, dat: &Vec<f64>) -> Self {
        let mut cols = vec![vec![]; n_row];
        let mut data = vec![vec![]; n_row];
        for ((&r, &c), &d) in row.iter().zip(col).zip(dat) {
            cols[r].push(c);
            data[r].push(d);
        }
        Self { cols, data }
    }
    pub fn dot(&self, rhs: &Array1<f64>) -> Array1<f64> {
        return self
            .cols
            .iter()
            .zip(&self.data)
            .map(|(col, dat)| col.iter().zip(dat.iter()).map(|(&c, d)| rhs[c] * d).sum())
            .collect::<Vec<_>>()
            .into();
    }
}

pub struct Game {
    pub sp1: StrategyPolytope,
    pub sp2: StrategyPolytope,
    pub mat_a: SparseMatrix,
    pub mat_a_t: SparseMatrix,
}
impl Game {
    pub fn load(filepath: &str) -> Self {
        let json = {
            let file_content = std::fs::read_to_string(filepath).unwrap();
            serde_json::from_str::<Value>(&file_content).unwrap()
        };
        let sp1 = StrategyPolytope {
            par: json["x"]["par"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_u64().unwrap() as usize)
                .collect(),
            idx: json["x"]["idx"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_u64().unwrap() as usize)
                .collect(),
        };
        let sp2 = StrategyPolytope {
            par: json["y"]["par"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_u64().unwrap() as usize)
                .collect(),
            idx: json["y"]["idx"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_u64().unwrap() as usize)
                .collect(),
        };
        let row: Vec<usize> = json["A"]["row"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_u64().unwrap() as usize)
            .collect();
        let col: Vec<usize> = json["A"]["col"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_u64().unwrap() as usize)
            .collect();
        let data: Vec<f64> = json["A"]["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        let mat_a = SparseMatrix::new(*sp1.idx.last().unwrap(), &row, &col, &data);
        let mat_a_t = SparseMatrix::new(*sp2.idx.last().unwrap(), &col, &row, &data);
        Self {
            sp1,
            sp2,
            mat_a,
            mat_a_t,
        }
    }
    pub fn error(&self, x: &Array1<f64>, y: &Array1<f64>) -> f64 {
        let min: f64 = self.sp1.minimize(self.mat_a.dot(y));
        let max: f64 = self.sp2.maximize(self.mat_a_t.dot(x));
        max - min
    }
}
