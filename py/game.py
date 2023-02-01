from __future__ import annotations

import copy
from enum import IntEnum
from typing import Generic, Hashable, Optional, TypeVar


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

    def obs(self) -> O:
        # represents the information partition
        raise NotImplementedError

    def payoff(self) -> float:
        # gain for Player.P1, loss for Player.P2
        raise NotImplementedError

    def expected_value(
        self, x: dict[O, dict[A, float]], y: dict[O, dict[A, float]], prob: float = 1.0
    ) -> float:
        player = self.player()
        if player is None:
            return self.payoff() * prob

        total = 0.0
        for action in self.legal_actions():
            if player == Player.P1:
                p = x[self.obs()][action]
            if player == Player.P2:
                p = y[self.obs()][action]
            if player == Player.C:
                p = self.prob(action)
            s = copy.deepcopy(self)
            s.step(action)
            total += s.expected_value(x, y, prob * p)
        return total
