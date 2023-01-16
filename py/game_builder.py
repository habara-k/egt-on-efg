from __future__ import annotations

import copy
import json
from collections import defaultdict
from enum import IntEnum
from typing import DefaultDict, Generic, Hashable, Optional, TypeVar


class Player(IntEnum):
    P1 = 0
    P2 = 1
    C = 2

    @staticmethod
    def opponent(player: Player) -> Player:
        assert player is not Player.C
        if player is Player.P1:
            return Player.P2
        return Player.P1


A = TypeVar("A", bound=Hashable)
O = TypeVar("O", bound=Hashable)


class GameState(Generic[A, O]):
    def player(self) -> Optional[Player]:
        raise NotImplementedError

    def legal_actions(self) -> list[A]:
        raise NotImplementedError

    def step(self, action: A):
        raise NotImplementedError

    def prob(self, action: A) -> float:
        # used only when self.player() == Player.C
        raise NotImplementedError

    def obs(self, player: Player) -> O:
        # represents the information partition
        raise NotImplementedError

    def payoff(self) -> float:
        # gain for Player.P1, loss for Player.P2
        raise NotImplementedError


class GameBuilder(Generic[A, O]):
    def __init__(self, state: GameState[A, O]):
        self.par: tuple[list[int], list[int]] = ([], [])
        self.idx: tuple[list[int], list[int]] = ([1], [1])
        self.obs: tuple[dict[O, int], dict[O, int]] = ({}, {})
        self.action: tuple[list[list[A]], list[list[A]]] = ([], [])
        self.payoff_dict: DefaultDict[tuple[int, int], float] = defaultdict(float)
        self.build(state, 1.0, (0, 0))

    def build(
        self,
        state: GameState[A, O],
        prob: float,
        parent: tuple[int, int],
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

        observation: O = state.obs(player)
        if observation not in self.obs[player]:
            self.obs[player][observation] = len(self.obs[player])
            legal_actions = state.legal_actions()
            self.action[player].append(legal_actions)
            n = len(legal_actions)
            self.par[player].append(parent[player])
            self.idx[player].append(self.idx[player][-1] + n)

        obs: int = self.obs[player][observation]

        assert state.legal_actions() == self.action[player][obs]
        for i, action in enumerate(self.action[player][obs]):
            s = copy.deepcopy(state)
            s.step(action)
            self.build(
                s,
                prob,
                (self.idx[0][obs] + i, parent[1])
                if player is Player.P1
                else (parent[0], self.idx[1][obs] + i),
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
            row.append(r)
            col.append(c)
            data.append(-val)
        return {
            "row": row,
            "col": col,
            "data": data,
        }

    def sp(i: int):
        tmp = sorted([(i, obs) for obs, i in game.obs[i].items()])
        obs = [obs for i, obs in tmp]
        return {
            "par": game.par[i],
            "idx": game.idx[i],
            "obs": obs,
            "action": game.action[i],
        }

    obj = {
        "x": sp(0),
        "y": sp(1),
        "A": mat_a(),
    }

    return json.dumps(obj)
