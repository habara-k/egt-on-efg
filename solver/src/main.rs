extern crate blas_src;

use solver::cfr::{cfr, cfr_plus};
use solver::egt::EGT;
use solver::game::Game;

use argh::FromArgs;
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
    if cfg.method == "cfr" {
        cfr(&game, cfg.steps);
    }
    if cfg.method == "cfr+" {
        cfr_plus(&game, cfg.steps);
    }
    if cfg.method == "egt" {
        let egt = EGT::new(&game);
        egt.run(cfg.steps);
    }
}
