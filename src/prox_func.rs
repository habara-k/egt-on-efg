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
}

pub struct Normal<'a> {
    sp: &'a StrategyPolytope,
    w: Array1<f64>,
    c: Array1<f64>, // the coefficient of `x\lnx`
    _center: Array1<f64>,
    min: f64,
}

impl<'a> Normal<'a> {
    pub fn new(sp: &'a StrategyPolytope) -> Self {
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
        let min: f64 = -conj(&sp, &mut _center, &w);

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
        conj(&self.sp, &mut x, &self.w) + self.min
    }
    fn conj_grad(&self, mut x: Array1<f64>) -> Array1<f64> {
        // Return ∇d*(x)
        conj(&self.sp, &mut x, &self.w);
        x
    }
    fn center(&self) -> &Array1<f64> {
        &self._center
    }
}

pub struct Centering<'a> {
    sp: &'a StrategyPolytope,
    w: Array1<f64>,
    c0: Array1<f64>, // the coefficient of `x\lnx`
    c1: Array1<f64>, // the coefficient of `x`
    _center: Array1<f64>,
    min: f64,
}

impl<'a> Centering<'a> {
    pub fn new(sp: &'a StrategyPolytope, _center: Array1<f64>) -> Self {
        let mut w: Array1<f64> = Array1::ones(sp.par.len());
        let mut ws: Array1<f64> = Array1::zeros(*sp.idx.last().unwrap());
        for (i, &p) in sp.par.iter().enumerate().rev() {
            w[i] += (sp.idx[i]..sp.idx[i + 1])
                .map(|j| ws[j])
                .fold(f64::NEG_INFINITY, |m, v| v.max(m));
            ws[p] += w[i]
        }
        let mut c0: Array1<f64> = -ws;
        for i in 0..sp.par.len() {
            for j in sp.idx[i]..sp.idx[i + 1] {
                c0[j] += w[i];
            }
        }
        let c1 = -(_center.mapv(f64::ln) + 1.0) * &c0;
        let min = _center.mapv(|v| v * v.ln()).dot(&c0) + c1.dot(&_center);

        Self {
            sp,
            w,
            c0,
            c1,
            _center,
            min,
        }
    }
}
impl ProxFunction for Centering<'_> {
    fn grad(&self, x: Array1<f64>) -> Array1<f64> {
        // Return ∇d(x)
        (x.mapv(f64::ln) + 1.0) * &self.c0 + &self.c1
    }
    fn conj(&self, mut x: Array1<f64>) -> f64 {
        // Return d(x)
        x -= &self.c1;
        conj(&self.sp, &mut x, &self.w) + self.min
    }
    fn conj_grad(&self, mut x: Array1<f64>) -> Array1<f64> {
        // Return ∇d*(x)
        x -= &self.c1;
        conj(&self.sp, &mut x, &self.w);
        x
    }
    fn center(&self) -> &Array1<f64> {
        &self._center
    }
}

pub struct Farina2021<'a> {
    sp: &'a StrategyPolytope,
    w: Array1<f64>,
    c0: Array1<f64>, // the coefficient of `x\lnx`
    c1: Array1<f64>, // the coefficient of `x`
    _center: Array1<f64>,
}

impl<'a> Farina2021<'a> {
    pub fn new(sp: &'a StrategyPolytope) -> Self {
        let mut w: Array1<f64> = Array1::ones(sp.par.len());
        let mut ws: Array1<f64> = Array1::zeros(*sp.idx.last().unwrap());
        for (i, &p) in sp.par.iter().enumerate().rev() {
            w[i] += (sp.idx[i]..sp.idx[i + 1])
                .map(|j| ws[j])
                .fold(f64::NEG_INFINITY, |m, v| v.max(m));
            ws[p] += w[i]
        }
        let mut c0: Array1<f64> = -ws;
        for i in 0..sp.par.len() {
            for j in sp.idx[i]..sp.idx[i + 1] {
                c0[j] += w[i];
            }
        }
        let mut c1 = Array1::zeros(*sp.idx.last().unwrap());
        for (i, &p) in sp.par.iter().enumerate() {
            c1[p] += w[i] * ((sp.idx[i + 1] - sp.idx[i]) as f64).ln();
        }
        let mut _center = -c1.clone();
        assert!(conj(&sp, &mut _center, &w).abs() < 1e-9);

        Self {
            sp,
            w,
            c0,
            c1,
            _center,
        }
    }
}
impl ProxFunction for Farina2021<'_> {
    fn grad(&self, x: Array1<f64>) -> Array1<f64> {
        // Return ∇d(x)
        (x.mapv(f64::ln) + 1.0) * &self.c0 + &self.c1
    }
    fn conj(&self, mut x: Array1<f64>) -> f64 {
        // Return d(x)
        x -= &self.c1;
        conj(&self.sp, &mut x, &self.w)
    }
    fn conj_grad(&self, mut x: Array1<f64>) -> Array1<f64> {
        // Return ∇d*(x)
        x -= &self.c1;
        conj(&self.sp, &mut x, &self.w);
        x
    }
    fn center(&self) -> &Array1<f64> {
        &self._center
    }
}

fn conj(sp: &StrategyPolytope, x: &mut Array1<f64>, w: &Array1<f64>) -> f64 {
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
    let val: f64 = x[0];
    x[0] = 1.0;
    for (i, &p) in sp.par.iter().enumerate() {
        let xp = x[p];
        for j in sp.idx[i]..sp.idx[i + 1] {
            x[j] *= xp;
        }
    }
    val
}
