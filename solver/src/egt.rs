use crate::game::{Game, StrategyPolytope};
use ndarray::{s, Array1};
use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

struct ProxFunction<'a> {
    sp: &'a StrategyPolytope,
    w: Array1<f64>,
    c: Array1<f64>,
    center: Array1<f64>,
    min: f64,
}

impl<'a> ProxFunction<'a> {
    fn new(sp: &'a StrategyPolytope) -> Self {
        let mut w: Array1<f64> = Array1::ones(sp.par.len());
        let mut ws: Array1<f64> = Array1::zeros(*sp.idx.last().unwrap());
        for (i, &p) in sp.par.iter().enumerate().rev() {
            w[i] += ws
                .slice(s![sp.idx[i]..sp.idx[i + 1]])
                .fold(f64::NEG_INFINITY, |m, v| v.max(m));
            ws[p] += w[i]
        }
        let mut c: Array1<f64> = -ws;
        for i in 0..sp.par.len() {
            c.slice_mut(s![sp.idx[i]..sp.idx[i + 1]]).add_assign(w[i]);
        }
        let mut center: Array1<f64> = Array1::zeros(*sp.idx.last().unwrap());
        let min: f64 = -conj(&sp, &mut center, &w, 0.0);

        Self {
            sp,
            w,
            c,
            center,
            min,
        }
    }
    fn grad(&self, x: Array1<f64>) -> Array1<f64> {
        // Return ∇d(x)
        (x.mapv(f64::ln) + 1.0) * &self.c
    }
    fn conj(&self, mut x: Array1<f64>) -> f64 {
        // Return d(x)
        conj(&self.sp, &mut x, &self.w, self.min)
    }
    fn conj_grad(&self, mut x: Array1<f64>) -> Array1<f64> {
        // Return ∇d*(x)
        conj(&self.sp, &mut x, &self.w, self.min);
        x
    }
    fn projection(&self, x: Array1<f64>, s: Array1<f64>) -> Array1<f64> {
        // Return ∇d*(∇d(z)-s)
        self.conj_grad(self.grad(x) - s)
    }
}

fn conj(sp: &StrategyPolytope, x: &mut Array1<f64>, w: &Array1<f64>, min: f64) -> f64 {
    // x ← ∇d*(x)
    // Return d*(x)
    for (i, &p) in sp.par.iter().enumerate().rev() {
        let l: usize = sp.idx[i];
        let r: usize = sp.idx[i + 1];
        x.slice_mut(s![l..r]).div_assign(w[i]);
        let max = x.slice(s![l..r]).fold(0.0 / 0.0, |m, v| v.max(m));
        if max == f64::NEG_INFINITY {
            x.slice_mut(s![l..r]).fill(1.0 / (r - l) as f64);
            x[p] = f64::NEG_INFINITY;
            continue;
        }
        x.slice_mut(s![l..r]).sub_assign(max);
        x.slice_mut(s![l..r]).mapv_inplace(f64::exp);
        let z: f64 = x.slice(s![l..r]).sum();
        x.slice_mut(s![l..r]).div_assign(z);
        x[p] += (z.ln() + max) * w[i];
    }
    let val: f64 = x[0] + min;
    x[0] = 1.0;
    for (i, &p) in sp.par.iter().enumerate() {
        let xp = x[p];
        x.slice_mut(s![sp.idx[i]..sp.idx[i + 1]]).mul_assign(xp);
    }
    val
}

pub struct EGT<'a> {
    game: &'a Game,
    pf1: ProxFunction<'a>,
    pf2: ProxFunction<'a>,
}
impl<'a> EGT<'a> {
    pub fn new(game: &'a Game) -> Self {
        let pf1 = ProxFunction::new(&game.sp1);
        let pf2 = ProxFunction::new(&game.sp2);
        Self { game, pf1, pf2 }
    }
    fn excessive_gap(&self, x: &Array1<f64>, y: &Array1<f64>, mu1: f64, mu2: f64) -> f64 {
        let phi: f64 = -mu1 * self.pf1.conj(self.game.mat_a.dot(y) / -mu1);
        let f: f64 = mu2 * self.pf2.conj(self.game.mat_a_t.dot(x) / mu2);
        phi - f
    }
    fn init(&self, mu1: f64, mu2: f64) -> (Array1<f64>, Array1<f64>) {
        let y = self
            .pf2
            .conj_grad(self.game.mat_a_t.dot(&self.pf1.center) / mu2);
        let x = self
            .pf1
            .projection(self.pf1.center.clone(), self.game.mat_a.dot(&y) / mu1);
        (x, y)
    }
    fn initialize(&self) -> (Array1<f64>, Array1<f64>, f64) {
        let mut mu: f64 = 1e-6;
        loop {
            let (x, y) = self.init(mu, mu);
            if self.excessive_gap(&x, &y, mu, mu) > 0.0 {
                return (x, y, mu);
            }
            mu *= 1.2;
            assert!(mu < 1e9);
        }
    }
    fn shrink_mu1(
        &self,
        x: &Array1<f64>,
        y: &Array1<f64>,
        mu1: f64,
        mu2: f64,
        tau: f64,
    ) -> (Array1<f64>, Array1<f64>, f64) {
        let x_opt = self.pf1.conj_grad(self.game.mat_a.dot(y) / -mu1);
        let x_hat = (1.0 - tau) * x + tau * &x_opt;
        let y_opt = self.pf2.conj_grad(self.game.mat_a_t.dot(&x_hat) / mu2);
        let x_tilde = self.pf1.projection(
            x_opt,
            self.game.mat_a.dot(&y_opt) * (tau / ((1.0 - tau) * mu1)),
        );
        (
            (1.0 - tau) * x + tau * &x_tilde,
            (1.0 - tau) * y + tau * &y_opt,
            (1.0 - tau) * mu1,
        )
    }
    fn shrink_mu2(
        &self,
        x: &Array1<f64>,
        y: &Array1<f64>,
        mu1: f64,
        mu2: f64,
        tau: f64,
    ) -> (Array1<f64>, Array1<f64>, f64) {
        let y_opt = self.pf2.conj_grad(self.game.mat_a_t.dot(x) / mu2);
        let y_hat = (1.0 - tau) * y + tau * &y_opt;
        let x_opt = self.pf1.conj_grad(self.game.mat_a.dot(&y_hat) / -mu1);
        let y_tilde = self.pf2.projection(
            y_opt,
            self.game.mat_a_t.dot(&x_opt) * (-tau / ((1.0 - tau) * mu2)),
        );
        (
            (1.0 - tau) * x + tau * &x_opt,
            (1.0 - tau) * y + tau * &y_tilde,
            (1.0 - tau) * mu2,
        )
    }
    fn decrease_mu1(
        &self,
        x: &Array1<f64>,
        y: &Array1<f64>,
        mu1: f64,
        mu2: f64,
        mut tau: f64,
    ) -> (Array1<f64>, Array1<f64>, f64, f64) {
        loop {
            let (nxt_x, nxt_y, nxt_mu1) = self.shrink_mu1(x, y, mu1, mu2, tau);
            if self.excessive_gap(&nxt_x, &nxt_y, nxt_mu1, mu2) > 0.0 {
                return (nxt_x, nxt_y, nxt_mu1, tau);
            }
            tau *= 0.5;
            assert!(tau > 1e-20);
        }
    }
    fn decrease_mu2(
        &self,
        x: &Array1<f64>,
        y: &Array1<f64>,
        mu1: f64,
        mu2: f64,
        mut tau: f64,
    ) -> (Array1<f64>, Array1<f64>, f64, f64) {
        loop {
            let (nxt_x, nxt_y, nxt_mu2) = self.shrink_mu2(x, y, mu1, mu2, tau);
            if self.excessive_gap(&nxt_x, &nxt_y, mu1, nxt_mu2) > 0.0 {
                return (nxt_x, nxt_y, nxt_mu2, tau);
            }
            tau *= 0.5;
            assert!(tau > 1e-20);
        }
    }
    pub fn run(&self, step: usize) {
        let start = std::time::Instant::now();
        let (mut x, mut y, mu) = self.initialize();
        dbg!(&mu);
        let mut mu1: f64 = mu;
        let mut mu2: f64 = mu;
        let mut tau: f64 = 0.5;

        let mut result = vec![self.game.error(&x, &y)];
        for _ in 0..step {
            while self.excessive_gap(&x, &y, mu1 * 0.9, mu2 * 0.9) > 0.0 {
                mu1 *= 0.9;
                mu2 *= 0.9;
            }
            if mu1 > mu2 {
                (x, y, mu1, tau) = self.decrease_mu1(&x, &y, mu1, mu2, tau)
            } else {
                (x, y, mu2, tau) = self.decrease_mu2(&x, &y, mu1, mu2, tau)
            }
            assert!(self.excessive_gap(&x, &y, mu1, mu2) >= 0.0);
            result.push(self.game.error(&x, &y))
        }
        dbg!(&result[0]);
        dbg!(&result[step]);
        let end = start.elapsed();
        println!(
            "{}.{:03}[s] elapsed.",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
    }
}
