import datetime
import json

import matplotlib.pyplot as plt  # type: ignore
import numpy as np
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


def travel(x_path: str, game_path: str, player: str):
    assert player == "x" or player == "y"
    with open(f"{x_path}/{player}.json", "r") as f:
        x = np.array(json.load(f))
    with open(game_path, "r") as f:
        game = json.load(f)[player]

    inv: list[list[int]] = [[] for i in range(game["idx"][-1])]
    for i, p in enumerate(game["par"]):
        inv[p].append(i)

    path = [0]

    while True:
        if len(path) % 2 == 1:
            j = path[-1]
            obs = [game["obs"][i] for i in inv[j]]
            print(f"{obs=}")
            k = int(input("Select observation: "))
            if 0 <= k < len(obs):
                path.append(inv[j][k])
            elif len(path) > 1:
                path.pop()

        else:
            i = path[-1]
            l = game["idx"][i]
            r = game["idx"][i + 1]
            action = game["action"][i]
            print(f"{action=}")
            print(f"{x[l:r] / x[l:r].sum()}")
            a = int(input("Select action: "))
            if 0 <= a < r - l:
                path.append(l + a)
            else:
                path.pop()


Fire(
    {
        "build": build,
        "draw": draw,
        "draw_multi": draw_multi,
        "travel": travel,
    }
)
