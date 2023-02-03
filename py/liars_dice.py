import math
from typing import Optional

from game import GameState, Player

Card = str
Action = Card
Obs = str

FACE_NUM = 6


class LiarsDice(GameState[Action, Obs]):
    def __init__(self, dice_num: int = 1):
        self.DICE_NUM = dice_num
        self.HAND: list[str] = []

        def dfs(hand: str, s: int):
            if len(hand) == self.DICE_NUM:
                self.HAND.append(hand)
                return
            for i in range(s, FACE_NUM + 1):
                h = hand[:]
                h += str(i)
                dfs(h, i)

        dfs("", 1)

        self.hand: Optional[tuple[str, str]] = None
        self.history: list[str] = []
        self.bid: Optional[tuple[int, int]] = None

    def player(self) -> Optional[Player]:
        if len(self.history) > 0 and self.history[-1] == "liar":
            return None
        if self.hand is None:
            return Player.C
        if len(self.history) % 2 == 0:
            return Player.P2
        return Player.P1

    def legal_actions(self) -> list[Action]:
        player = self.player()
        assert player is not None
        if player is Player.C:
            return [f"{h1}:{h2}" for h1 in self.HAND for h2 in self.HAND]
        if self.bid is None:
            return [
                f"{face}:{num}"
                for face in range(1, FACE_NUM + 1)
                for num in range(1, 2 * self.DICE_NUM + 1)
            ]

        f, n = self.bid
        return [
            f"{face}:{num}"
            for face in range(f, FACE_NUM + 1)
            for num in range(1, 2 * self.DICE_NUM + 1)
            if f < face or n < num
        ] + ["liar"]

    def step(self, action: Action):
        # assert action in self.legal_actions()
        player = self.player()
        assert player is not None

        if player is Player.C:
            h1: str = action[: self.DICE_NUM]
            h2: str = action[self.DICE_NUM + 1 :]
            self.hand = (h1, h2)
        elif action == "liar":
            self.terminated = True
        else:
            face = int(action[0])
            num = int(action[2:])
            self.bid = (face, num)

        self.history.append(action)

    def prob(self, action: Action) -> float:
        assert self.player() is Player.C
        h1: str = action[: self.DICE_NUM]
        h2: str = action[self.DICE_NUM + 1 :]
        p = (1.0 / FACE_NUM) ** self.DICE_NUM
        p *= math.factorial(self.DICE_NUM) ** 2
        for c in set(h1):
            p /= math.factorial(h1.count(c))
        for c in set(h2):
            p /= math.factorial(h2.count(c))
        return p

    def obs(self) -> Obs:
        player = self.player()
        assert player is not None
        assert player is not Player.C
        assert self.hand is not None
        return ",".join([self.hand[player], *self.history[1:]])

    def payoff(self) -> float:
        assert self.player() is None
        assert self.bid is not None
        face, num = self.bid
        assert self.hand is not None
        bid_is_valid = (
            self.hand[0].count(str(face)) + self.hand[1].count(str(face)) >= num
        )
        bid_is_by_p1 = len(self.history) % 2 == 1

        if bid_is_valid == bid_is_by_p1:
            return -1
        return 1
