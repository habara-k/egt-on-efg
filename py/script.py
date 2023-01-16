import datetime
import json

import matplotlib.pyplot as plt  # type: ignore
from fire import Fire  # type: ignore

import game_builder
from kuhn import KuhnPoker
from leduc import LeducHoldem


def build(key: str, path: str):
    def game(key: str):
        if key == "kuhn":
            return KuhnPoker()
        elif key == "leduc":
            return LeducHoldem()
        elif key == "leduc13":
            return LeducHoldem(
                ["2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K", "A"]
            )
        else:
            assert False, f"Invalid key: {key}"

    with open(path, "w") as f:
        f.write(game_builder.build(game(key)))


def draw(path: str):
    with open(f"{path}/error.json", "r") as f:
        error = json.load(f)
    plt.plot(error)
    plt.xscale("log")
    plt.yscale("log")
    plt.xlabel("iterations")
    plt.ylabel("error")
    plt.grid()
    plt.savefig(f"{path}/error.png")


def draw_multi(logdir: str, game: str, *args):
    for i in range(0, len(args), 2):
        path = args[i]
        label = args[i + 1]
        with open(f"{path}/error.json", "r") as f:
            error = json.load(f)
        plt.plot(error, label=label)
    plt.title(game)
    plt.xscale("log")
    plt.yscale("log")
    plt.xlabel("iterations")
    plt.ylabel("error")
    plt.grid()
    plt.legend()
    now = datetime.datetime.now().strftime("%Y%m%d-%H:%M")
    plt.savefig(f"{logdir}/{now}-{game}-error.png")


Fire({"build": build, "draw": draw, "draw_multi": draw_multi})
