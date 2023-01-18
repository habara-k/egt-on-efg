use crate::game::Game;
use crate::prox_func::{build_prox_function, ProxFunction};
use indicatif::ProgressIterator;
use ndarray::Array1;

pub struct EGT<'a> {
    game: &'a Game,
    pf1: Box<dyn ProxFunction + 'a>,
    pf2: Box<dyn ProxFunction + 'a>,
}
impl<'a> EGT<'a> {
    pub fn new(game: &'a Game, pf_key: &str) -> Self {
        let pf1 = build_prox_function(pf_key, &game.sp1);
        let pf2 = build_prox_function(pf_key, &game.sp2);
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
            .conj_grad(self.game.mat_a_t.dot(&self.pf1.center()) / mu2);
        let x = self
            .pf1
            .projection(self.pf1.center().clone(), self.game.mat_a.dot(&y) / mu1);
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
    pub fn set_center(&mut self, x: Array1<f64>, y: Array1<f64>) {
        self.pf1.set_center(x);
        self.pf2.set_center(y);
    }
    pub fn run(&self, step: usize) -> (Array1<f64>, Array1<f64>, Vec<f64>) {
        let (mut x, mut y, mu) = self.initialize();
        dbg!(&mu);
        let mut mu1: f64 = mu;
        let mut mu2: f64 = mu;
        let mut tau: f64 = 0.5;

        let mut error = vec![self.game.error(&x, &y)];
        dbg!(&error[0]);

        for _ in (1..step).progress() {
            if mu1 > mu2 {
                (x, y, mu1, tau) = self.decrease_mu1(&x, &y, mu1, mu2, tau);
            } else {
                (x, y, mu2, tau) = self.decrease_mu2(&x, &y, mu1, mu2, tau);
            }
            // assert!(self.excessive_gap(&x, &y, mu1, mu2) >= 0.0);
            error.push(self.game.error(&x, &y));
        }
        dbg!(&mu1);
        dbg!(&mu2);
        dbg!(&error[step - 1]);
        (x, y, error)
    }
}
