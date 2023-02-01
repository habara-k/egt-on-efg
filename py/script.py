import datetime
import json

import matplotlib.pyplot as plt  # type: ignore
import numpy as np
from fire import Fire  # type: ignore

import game_builder
from kuhn import KuhnPoker
from leduc import LeducHoldem


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


def build(key: str, path: str):
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


MARKER = ["^", "s", "D", "o", "*"]
MARKERSIZE = [8, 8, 7, 8, 11]
COLOR = ["#E69F00", "#56B4E9", "#009E73", "#0072B2", "#D55E00", "#CC79A7", "#F0E442"]


def draw_multi(logdir: str, game: str, *args):
    plt.rcParams["font.family"] = "Times New Roman"
    plt.rcParams["font.size"] = 15
    plt.rcParams["figure.figsize"] = (8, 4)

    for i in range(0, len(args) // 2):
        path = args[2 * i]
        label = args[2 * i + 1]
        with open(f"{path}/error.json", "r") as f:
            error = json.load(f)

        if label in ["EGT-centering", "EGT-centering with CFR+"]:
            start = len(error) // 10
        else:
            start = len(error) // 1000

        markevery = [0, len(error) - 1 - start]

        iter = list(range(start, len(error)))
        plt.plot(
            iter,
            error[start:],
            label=label,
            linewidth=1.0,
            color=COLOR[i],
            markersize=MARKERSIZE[i],
            marker=MARKER[i],
            markevery=markevery,
            markeredgewidth=0.5,
            markeredgecolor="k",
        )

    plt.ylabel("Error")
    plt.xlabel("Iteration")
    plt.xscale("log")
    plt.yscale("log")

    plt.grid()
    plt.legend(loc="lower left")

    plt.title(game)
    plt.tight_layout()
    now = datetime.datetime.now().strftime("%Y%m%d-%H:%M")
    plt.savefig(f"{logdir}/{now}-{game}-error.png", dpi=300)


def strategy(path: str, game_path: str):
    for player in ["x", "y"]:
        with open(f"{path}/{player}.json", "r") as f:
            x = np.array(json.load(f))
        with open(game_path, "r") as f:
            game = json.load(f)[player]

        obj = {}
        for i in range(len(game["obs"])):
            l = game["idx"][i]
            r = game["idx"][i + 1]
            p = game["par"][i]
            strt = dict(zip(game["action"][i], np.array(x[l:r]) / x[p]))
            obj[game["obs"][i]] = strt

        with open(f"{path}/{player}-strt.json", "w") as f:
            f.write(json.dumps(obj))


def expected_value(path: str, game_key: str):
    with open(f"{path}/x-strt.json", "r") as f:
        x = json.load(f)
    with open(f"{path}/y-strt.json", "r") as f:
        y = json.load(f)
    print(game(game_key).expected_value(x, y))


Fire(
    {
        "build": build,
        "draw": draw,
        "draw_multi": draw_multi,
        "strategy": strategy,
        "ev": expected_value,
    }
)
