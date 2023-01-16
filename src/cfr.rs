use crate::game::{Game, StrategyPolytope};
use indicatif::ProgressIterator;
use ndarray::{s, Array1};
use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

fn prod(sp: &StrategyPolytope, mut x: Array1<f64>) -> Array1<f64> {
    x[0] = 1.0;
    for (i, &p) in sp.par.iter().enumerate() {
        let xp = x[p];
        x.slice_mut(s![sp.idx[i]..sp.idx[i + 1]]).mul_assign(xp);
    }
    x
}
fn accumulate(
    sp: &StrategyPolytope,
    x: &Array1<f64>,
    mut util: Array1<f64>,
    regret: &mut Array1<f64>,
) {
    for (i, &p) in sp.par.iter().enumerate().rev() {
        let l = sp.idx[i];
        let r = sp.idx[i + 1];
        let avg = util.slice(s![l..r]).dot(&x.slice(s![l..r]));
        regret.slice_mut(s![l..r]).sub_assign(avg);
        regret.slice_mut(s![l..r]).add_assign(&util.slice(s![l..r]));
        util[p] += avg;
    }
}
fn normalize(sp: &StrategyPolytope, regret: Array1<f64>) -> Array1<f64> {
    let mut x = regret;
    x[0] = 1.0;
    for i in 0..(sp.idx.len() - 1) {
        let l = sp.idx[i];
        let r = sp.idx[i + 1];
        let total = x.slice(s![l..r]).sum();
        if total == 0.0 {
            x.slice_mut(s![l..r]).fill(1.0 / (r - l) as f64);
        } else {
            x.slice_mut(s![l..r]).div_assign(total);
        }
    }
    x
}

pub fn cfr(game: &Game, steps: usize) -> (Array1<f64>, Array1<f64>, Vec<f64>) {
    let mut regret_x = Array1::<f64>::zeros(*game.sp1.idx.last().unwrap());
    let mut regret_y = Array1::<f64>::zeros(*game.sp2.idx.last().unwrap());
    let mut x = normalize(&game.sp1, regret_x.clone());
    let mut y = normalize(&game.sp2, regret_y.clone());
    let mut px = prod(&game.sp1, x.clone());
    let mut py = prod(&game.sp2, y.clone());
    let mut sum_px = px.clone();
    let mut sum_py = py.clone();

    let mut error = vec![game.error(&px, &py)];
    dbg!(&error[0]);
    for k in (1..(steps + 1)).progress() {
        accumulate(&game.sp1, &x, -game.mat_a.dot(&py), &mut regret_x);
        accumulate(&game.sp2, &y, game.mat_a_t.dot(&px), &mut regret_y);
        x = normalize(&game.sp1, regret_x.clone().mapv(|v| v.max(0.0)));
        y = normalize(&game.sp2, regret_y.clone().mapv(|v| v.max(0.0)));
        px = prod(&game.sp1, x.clone());
        py = prod(&game.sp2, y.clone());
        sum_px.add_assign(&px);
        sum_py.add_assign(&py);
        error.push(game.error(&sum_px, &sum_py) / (k + 1) as f64);
    }
    dbg!(&error[steps]);
    dbg!(&error.iter().fold(f64::INFINITY, |m, v| v.min(m)));
    (
        sum_px / (steps + 1) as f64,
        sum_py / (steps + 1) as f64,
        error,
    )
}

pub fn cfr_plus(game: &Game, steps: usize) -> (Array1<f64>, Array1<f64>, Vec<f64>) {
    let mut regret_x = Array1::<f64>::zeros(*game.sp1.idx.last().unwrap());
    let mut regret_y = Array1::<f64>::zeros(*game.sp2.idx.last().unwrap());
    let mut x = normalize(&game.sp1, regret_x.clone());
    let mut y = normalize(&game.sp2, regret_y.clone());
    let mut px = prod(&game.sp1, x.clone());
    let mut py = prod(&game.sp2, y.clone());
    let mut sum_px = px.clone();
    let mut sum_py = py.clone();

    let mut error = vec![game.error(&px, &py)];
    dbg!(&error[0]);
    for k in (1..(steps + 1)).progress() {
        accumulate(&game.sp1, &x, -game.mat_a.dot(&py), &mut regret_x);
        regret_x.mapv_inplace(|v| v.max(0.0));
        x = normalize(&game.sp1, regret_x.clone());
        px = prod(&game.sp1, x.clone());
        sum_px.add_assign(&((k + 1) as f64 * &px));

        accumulate(&game.sp2, &y, game.mat_a_t.dot(&px), &mut regret_y);
        regret_y.mapv_inplace(|v| v.max(0.0));
        y = normalize(&game.sp2, regret_y.clone());
        py = prod(&game.sp2, y.clone());
        sum_py.add_assign(&((k + 1) as f64 * &py));

        let weight = (k + 1) as f64 * (k + 2) as f64 / 2.0;
        error.push(game.error(&sum_px, &sum_py) / weight);
    }
    dbg!(&error[steps]);
    dbg!(&error.iter().fold(f64::INFINITY, |m, v| v.min(m)));
    let weight = (steps + 1) as f64 * (steps + 2) as f64 / 2.0;
    (sum_px / weight, sum_py / weight, error)
}
