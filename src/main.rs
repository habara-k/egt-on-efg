extern crate blas_src;

use egt_on_efg::cfr::{cfr, cfr_plus};
use egt_on_efg::egt::EGT;
use egt_on_efg::game::Game;

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
    steps: usize,
}

fn main() {
    let cfg: Config = argh::from_env();
    let game = Game::load(&cfg.game);
    let start = std::time::Instant::now();
    let (x, y, error) = match cfg.method.as_str() {
        "cfr" => cfr(&game, cfg.steps),
        "cfr+" => cfr_plus(&game, cfg.steps),
        "egt" => {
            let egt = EGT::new(&game, "normal");
            egt.run(cfg.steps)
        }
        "egt-centering" => {
            let egt = EGT::new(&game, "normal");
            let (x, y, mut error) = egt.run(cfg.steps / 10);
            let mut egt = EGT::new(&game, "centering");
            egt.set_center(x, y);
            let (x, y, mut error2) = egt.run(cfg.steps * 9 / 10);
            error.append(&mut error2);
            (x, y, error)
        }
        "mix" => {
            let (x, y, mut error) = cfr_plus(&game, cfg.steps / 10);
            let mut egt = EGT::new(&game, "centering");
            egt.set_center(x, y);
            let (x, y, mut error2) = egt.run(cfg.steps * 9 / 10);
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
