use crate::game::{Game, StrategyPolytope};
use indicatif::ProgressIterator;
use ndarray::Array1;
use std::ops::AddAssign;

fn prod(sp: &StrategyPolytope, mut x: Array1<f64>) -> Array1<f64> {
    x[0] = 1.0;
    for (i, &p) in sp.par.iter().enumerate() {
        let xp = x[p];
        for j in sp.idx[i]..sp.idx[i + 1] {
            x[j] *= xp;
        }
    }
    x
}
fn accumulate(
    sp: &StrategyPolytope,
    z: &Array1<f64>,
    mut util: Array1<f64>,
    regret: &mut Array1<f64>,
) {
    for (i, &p) in sp.par.iter().enumerate().rev() {
        let l = sp.idx[i];
        let r = sp.idx[i + 1];
        let avg: f64 = (l..r).map(|j| util[j] * z[j]).sum();
        for j in l..r {
            regret[j] += util[j] - avg;
        }
        util[p] += avg;
    }
}
fn normalize(sp: &StrategyPolytope, regret: Array1<f64>) -> Array1<f64> {
    let mut x = regret;
    x[0] = 1.0;
    for i in 0..(sp.idx.len() - 1) {
        let l = sp.idx[i];
        let r = sp.idx[i + 1];
        let total: f64 = (l..r).map(|j| x[j]).sum();
        if total == 0.0 {
            let v = 1.0 / (r - l) as f64;
            for j in l..r {
                x[j] = v;
            }
        } else {
            for j in l..r {
                x[j] /= total;
            }
        }
    }
    x
}

pub fn cfr(game: &Game, step: usize) -> (Array1<f64>, Array1<f64>, Vec<f64>) {
    let n = *game.sp1.idx.last().unwrap();
    let m = *game.sp2.idx.last().unwrap();
    let mut regret_x = Array1::<f64>::zeros(n);
    let mut regret_y = Array1::<f64>::zeros(m);
    let mut z_x = normalize(&game.sp1, regret_x.clone());
    let mut z_y = normalize(&game.sp2, regret_y.clone());
    let mut x = prod(&game.sp1, z_x.clone());
    let mut y = prod(&game.sp2, z_y.clone());
    let mut sum_x = x.clone();
    let mut sum_y = y.clone();

    let mut error = vec![game.error(&x, &y)];
    dbg!(error[0]);

    for k in (1..step).progress() {
        accumulate(&game.sp1, &z_x, -game.mat_a.dot(&y), &mut regret_x);
        accumulate(&game.sp2, &z_y, game.mat_a_t.dot(&x), &mut regret_y);
        z_x = normalize(&game.sp1, regret_x.clone().mapv(|v| v.max(0.0)));
        z_y = normalize(&game.sp2, regret_y.clone().mapv(|v| v.max(0.0)));
        x = prod(&game.sp1, z_x.clone());
        y = prod(&game.sp2, z_y.clone());
        sum_x.add_assign(&x);
        sum_y.add_assign(&y);
        error.push(game.error(&sum_x, &sum_y) / (k + 1) as f64);
    }
    dbg!(&error[step - 1]);
    (sum_x / step as f64, sum_y / step as f64, error)
}

pub fn cfr_plus(game: &Game, step: usize) -> (Array1<f64>, Array1<f64>, Vec<f64>) {
    let n = *game.sp1.idx.last().unwrap();
    let m = *game.sp2.idx.last().unwrap();
    let mut regret_x = Array1::<f64>::zeros(n);
    let mut regret_y = Array1::<f64>::zeros(m);
    let mut z_x = normalize(&game.sp1, regret_x.clone());
    let mut z_y = normalize(&game.sp2, regret_y.clone());
    let mut x = prod(&game.sp1, z_x.clone());
    let mut y = prod(&game.sp2, z_y.clone());
    let mut sum_x = x.clone();
    let mut sum_y = y.clone();

    let mut error = vec![game.error(&x, &y)];
    dbg!(error[0]);

    for k in (1..step).progress() {
        accumulate(&game.sp1, &z_x, -game.mat_a.dot(&y), &mut regret_x);
        regret_x.mapv_inplace(|v| v.max(0.0));
        z_x = normalize(&game.sp1, regret_x.clone());
        x = prod(&game.sp1, z_x.clone());
        sum_x.add_assign(&((k + 1) as f64 * &x));

        accumulate(&game.sp2, &z_y, game.mat_a_t.dot(&x), &mut regret_y);
        regret_y.mapv_inplace(|v| v.max(0.0));
        z_y = normalize(&game.sp2, regret_y.clone());
        y = prod(&game.sp2, z_y.clone());
        sum_y.add_assign(&((k + 1) as f64 * &y));

        let weight = (k + 1) as f64 * (k + 2) as f64 / 2.0;
        error.push(game.error(&sum_x, &sum_y) / weight);
    }

    dbg!(&error[step - 1]);
    let weight = step as f64 * (step + 1) as f64 / 2.0;
    (sum_x / weight, sum_y / weight, error)
}
