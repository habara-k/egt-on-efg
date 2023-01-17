use crate::game::StrategyPolytope;
use ndarray::Array1;

pub trait ProxFunction {
    fn grad(&self, x: Array1<f64>) -> Array1<f64>; // Return ∇d(x)
    fn conj(&self, x: Array1<f64>) -> f64; // Return d(x)
    fn conj_grad(&self, x: Array1<f64>) -> Array1<f64>; // Return ∇d*(x)
    fn projection(&self, x: Array1<f64>, s: Array1<f64>) -> Array1<f64> {
        // Return ∇d*(∇d(z)-s)
        self.conj_grad(self.grad(x) - s)
    }
    fn center(&self) -> &Array1<f64>;
    fn set_center(&mut self, x: Array1<f64>);
}

pub fn build_prox_function<'a>(key: &str, sp: &'a StrategyPolytope) -> Box<dyn ProxFunction + 'a> {
    if key == "normal" {
        return Box::new(Normal::new(&sp));
    }
    if key == "centering" {
        return Box::new(Centering::new(&sp));
    }
    panic!();
}

struct Normal<'a> {
    sp: &'a StrategyPolytope,
    w: Array1<f64>,
    c: Array1<f64>,
    _center: Array1<f64>,
    min: f64,
}

impl<'a> Normal<'a> {
    fn new(sp: &'a StrategyPolytope) -> Self {
        let mut w: Array1<f64> = Array1::ones(sp.par.len());
        let mut ws: Array1<f64> = Array1::zeros(*sp.idx.last().unwrap());
        for (i, &p) in sp.par.iter().enumerate().rev() {
            w[i] += (sp.idx[i]..sp.idx[i + 1])
                .map(|j| ws[j])
                .fold(f64::NEG_INFINITY, |m, v| v.max(m));
            ws[p] += w[i]
        }
        let mut c: Array1<f64> = -ws;
        for i in 0..sp.par.len() {
            for j in sp.idx[i]..sp.idx[i + 1] {
                c[j] += w[i];
            }
        }
        let mut _center: Array1<f64> = Array1::zeros(*sp.idx.last().unwrap());
        let min: f64 = -conj(&sp, &mut _center, &w, 0.0);

        Self {
            sp,
            w,
            c,
            _center,
            min,
        }
    }
}
impl ProxFunction for Normal<'_> {
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
    fn center(&self) -> &Array1<f64> {
        &self._center
    }
    fn set_center(&mut self, _x: Array1<f64>) {
        unimplemented!();
    }
}

struct Centering<'a> {
    pf: Normal<'a>,
    _center: Array1<f64>, // the new center
    min: f64,             // the value of `pf` at `_center`
    grad0: Array1<f64>,   // the gradient of `pf` at `_center`
}

impl<'a> Centering<'a> {
    fn new(sp: &'a StrategyPolytope) -> Self {
        let pf = Normal::new(&sp);
        let grad0 = Array1::zeros(*sp.idx.last().unwrap());
        let _center = pf.center().clone();
        let min = pf.min;
        Self {
            pf,
            grad0,
            _center,
            min,
        }
    }
}
impl ProxFunction for Centering<'_> {
    fn grad(&self, x: Array1<f64>) -> Array1<f64> {
        // Return ∇d(x)
        self.pf.grad(x) - &self.grad0
    }
    fn conj(&self, mut x: Array1<f64>) -> f64 {
        // Return d(x)
        x += &self.grad0;
        conj(&self.pf.sp, &mut x, &self.pf.w, self.min)
    }
    fn conj_grad(&self, mut x: Array1<f64>) -> Array1<f64> {
        // Return ∇d*(x)
        x += &self.grad0;
        conj(&self.pf.sp, &mut x, &self.pf.w, self.min);
        x
    }
    fn center(&self) -> &Array1<f64> {
        &self._center
    }
    fn set_center(&mut self, x: Array1<f64>) {
        self._center = x.clone();
        self.grad0 = self.pf.grad(x.clone());
        self.min = x.mapv(|v| v * v.ln()).dot(&self.pf.c) - self.grad0.dot(&x);
    }
}

fn conj(sp: &StrategyPolytope, x: &mut Array1<f64>, w: &Array1<f64>, min: f64) -> f64 {
    // x ← ∇d*(x)
    // Return d*(x)
    for (i, &p) in sp.par.iter().enumerate().rev() {
        let l: usize = sp.idx[i];
        let r: usize = sp.idx[i + 1];
        let max = (l..r)
            .map(|j| {
                x[j] /= w[i];
                x[j]
            })
            .fold(f64::NEG_INFINITY, |m, v| v.max(m));
        if max == f64::NEG_INFINITY {
            let v = 1.0 / (r - l) as f64;
            for j in l..r {
                x[j] = v;
            }
            x[p] = f64::NEG_INFINITY;
            continue;
        }
        let mut z = 0.0;
        for j in l..r {
            x[j] = (x[j] - max).exp();
            z += x[j];
        }
        for j in l..r {
            x[j] /= z;
        }
        x[p] += (z.ln() + max) * w[i];
    }
    let val: f64 = x[0] + min;
    x[0] = 1.0;
    for (i, &p) in sp.par.iter().enumerate() {
        let xp = x[p];
        for j in sp.idx[i]..sp.idx[i + 1] {
            x[j] *= xp;
        }
    }
    val
}
