from __future__ import annotations

import copy
import json
from collections import defaultdict
from typing import DefaultDict, Generic, Optional

from game import A, GameState, O, Player


class StrategySetBuilder:
    def __init__(self, parent: list[Optional[tuple[int, int]]], n_actions: list[int]):
        self.preorder: list[int] = []
        self.sequence: dict[Optional[tuple[int, int]], int] = {}
        self.par: list[int] = []
        self.idx: list[int] = [1]

        self.edge: DefaultDict[Optional[tuple[int, int]], list[int]] = defaultdict(list)
        for i, p in enumerate(parent):
            self.edge[p].append(i)

        def dfs(parent: Optional[tuple[int, int]], p: int):
            self.sequence[parent] = p
            for obs in self.edge[parent]:
                self.preorder.append(obs)
                self.par.append(p)
                l = self.idx[-1]
                n = n_actions[obs]
                self.idx.append(l + n)
                for i in range(n):
                    dfs((obs, i), l + i)

        dfs(None, 0)


class GameBuilder(Generic[A, O]):
    def __init__(self, state: GameState[A, O]):
        self.payoff_dict: DefaultDict[
            tuple[Optional[tuple[int, int]], Optional[tuple[int, int]]], float
        ] = defaultdict(float)
        self.obs: tuple[dict[O, int], dict[O, int]] = ({}, {})
        self.obs_list: tuple[list[O], list[O]] = ([], [])
        self.action: tuple[list[list[A]], list[list[A]]] = ([], [])
        self.parent: tuple[
            list[Optional[tuple[int, int]]], list[Optional[tuple[int, int]]]
        ] = ([], [])
        self.build(state, 1.0, (None, None))

        self.sp = (
            StrategySetBuilder(
                self.parent[0], list(map(lambda a: len(a), self.action[0]))
            ),
            StrategySetBuilder(
                self.parent[1], list(map(lambda a: len(a), self.action[1]))
            ),
        )

    def build(
        self,
        state: GameState[A, O],
        prob: float,
        parent: tuple[Optional[tuple[int, int]], Optional[tuple[int, int]]],
    ):
        player = state.player()
        if player is None:
            self.payoff_dict[parent] += state.payoff() * prob
            return

        if player is Player.C:
            for action in state.legal_actions():
                s = copy.deepcopy(state)
                s.step(action)
                self.build(s, prob * state.prob(action), parent)
            return

        observation: O = state.obs()
        if observation not in self.obs[player]:
            self.obs[player][observation] = len(self.obs[player])
            self.obs_list[player].append(observation)
            legal_actions = state.legal_actions()
            self.action[player].append(legal_actions)
            self.parent[player].append(parent[player])

        obs: int = self.obs[player][observation]

        assert state.legal_actions() == self.action[player][obs]
        for i, action in enumerate(self.action[player][obs]):
            s = copy.deepcopy(state)
            s.step(action)
            self.build(
                s,
                prob,
                ((obs, i), parent[1]) if player is Player.P1 else (parent[0], (obs, i)),
            )


def build(state: GameState[A, O]) -> str:
    game = GameBuilder(state)

    def mat_a():
        tmp = sorted([(key, val) for key, val in game.payoff_dict.items()])
        row = []
        col = []
        data = []
        for key, val in tmp:
            r, c = key
            if val == 0.0:
                continue
            row.append(game.sp[0].sequence[r])
            col.append(game.sp[1].sequence[c])
            data.append(val)
        return {
            "row": row,
            "col": col,
            "data": data,
        }

    def sp(player: int):
        obs = [game.obs_list[player][i] for i in game.sp[player].preorder]
        action = [game.action[player][i] for i in game.sp[player].preorder]

        return {
            "par": game.sp[player].par,
            "idx": game.sp[player].idx,
            "obs": obs,
            "action": action,
        }

    obj = {
        "x": sp(0),
        "y": sp(1),
        "A": mat_a(),
    }

    return json.dumps(obj)
