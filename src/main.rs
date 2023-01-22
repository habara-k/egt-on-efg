extern crate blas_src;

use egt_on_efg::cfr::{cfr, cfr_plus};
use egt_on_efg::egt::EGT;
use egt_on_efg::game::Game;
use egt_on_efg::prox_func::{Centering, Farina2021, Normal};

use argh::FromArgs;
use chrono::Local;
use std::io::Write;

#[derive(FromArgs)]
/// Config
struct Config {
    /// filepath of the game
    #[argh(option, short = 'g')]
    game: String,

    /// the method
    #[argh(option, short = 'm')]
    method: String,

    /// the number of iterations
    #[argh(option, short = 's')]
    step: usize,
}

fn main() {
    let cfg: Config = argh::from_env();
    let game = Game::load(&cfg.game);
    let start = std::time::Instant::now();
    let (x, y, error) = match cfg.method.as_str() {
        "cfr" => cfr(&game, cfg.step),
        "cfr+" => cfr_plus(&game, cfg.step),
        "egt" => {
            let pf1 = Normal::new(&game.sp1);
            let pf2 = Normal::new(&game.sp2);
            let egt = EGT::new(&game, &pf1, &pf2);
            egt.run(cfg.step)
        }
        "egt-farina" => {
            let pf1 = Farina2021::new(&game.sp1);
            let pf2 = Farina2021::new(&game.sp2);
            let egt = EGT::new(&game, &pf1, &pf2);
            egt.run(cfg.step)
        }
        "egt-centering" => {
            let pf1 = Normal::new(&game.sp1);
            let pf2 = Normal::new(&game.sp2);
            let egt = EGT::new(&game, &pf1, &pf2);
            let (x, y, mut error) = egt.run(cfg.step / 10);

            let pf1 = Centering::new(&game.sp1, x);
            let pf2 = Centering::new(&game.sp2, y);
            let egt = EGT::new(&game, &pf1, &pf2);
            let (x, y, mut error2) = egt.run(cfg.step * 9 / 10);
            error.append(&mut error2);
            (x, y, error)
        }
        "mix" => {
            let (x, y, mut error) = cfr_plus(&game, cfg.step / 10);
            let pf1 = Centering::new(&game.sp1, x);
            let pf2 = Centering::new(&game.sp2, y);
            let egt = EGT::new(&game, &pf1, &pf2);
            let (x, y, mut error2) = egt.run(cfg.step * 9 / 10);
            error.append(&mut error2);
            (x, y, error)
        }
        _ => panic!(),
    };
    let end = start.elapsed();
    println!(
        "{}.{:03}[s] elapsed.",
        end.as_secs(),
        end.subsec_nanos() / 1_000_000
    );

    let now = Local::now().format("%Y%m%d-%H:%M").to_string();
    let dirname = format!("log/{}-{}-{}", now, cfg.game, cfg.method);
    std::fs::create_dir_all(&dirname).unwrap();

    let mut file = std::fs::File::create(format!("{}/error.json", &dirname)).unwrap();
    writeln!(file, "{}", format!("{:?}", &error)).unwrap();

    let mut file = std::fs::File::create(format!("{}/x.json", &dirname)).unwrap();
    writeln!(file, "{}", format!("{}", &x)).unwrap();
    let mut file = std::fs::File::create(format!("{}/y.json", &dirname)).unwrap();
    writeln!(file, "{}", format!("{}", &y)).unwrap();
}
