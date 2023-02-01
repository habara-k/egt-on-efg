from typing import Optional

from game_builder import GameState, Player

Action = str
Obs = str
RANK = {"J": 0, "Q": 1, "K": 3}


class KuhnPoker(GameState[Action, Obs]):
    def __init__(self) -> None:
        self.terminated: bool = False
        self.bet: list[int] = [1, 1]
        self.history: list[Action] = []

    def player(self) -> Optional[Player]:
        if self.terminated:
            return None
        if not self.history:
            return Player.C
        if len(self.history) % 2 == 0:
            return Player.P2
        return Player.P1

    def legal_actions(self) -> list[Action]:
        assert self.player() is not None
        if not self.history:
            return [
                "JQ",
                "JK",
                "QJ",
                "QK",
                "KJ",
                "KQ",
            ]
        if self.history[-1] == "Bet":
            return ["Fold", "Call"]
        return ["Check", "Bet"]

    def step(self, action: Action):
        assert action in self.legal_actions()
        p = self.player()
        assert p is not None
        if action == "Bet":
            self.bet[p] = self.bet[Player.opponent(p)] + 1
        if action == "Check" and self.history[-1] == "Check":
            self.terminated = True
        if action == "Call":
            self.terminated = True
            self.bet[p] = self.bet[Player.opponent(p)]
        if action == "Fold":
            self.terminated = True
        self.history.append(action)

    def prob(self, action: Action) -> float:
        assert self.player() is Player.C
        return 1.0 / 6.0

    def obs(self) -> Obs:
        player = self.player()
        assert player is not None
        assert player is not Player.C
        hole_card = self.history[0][player]
        return ",".join([hole_card, *self.history[1:]])
        # return str((hole_card, self.history[1:]))

    def payoff(self) -> float:
        assert self.player() is None
        if self.history[-1] == "Fold":
            if len(self.history) % 2 == 0:
                return -self.bet[0]
            return self.bet[1]

        assert self.bet[0] == self.bet[1]
        bet = self.bet[0]
        card1, card2 = list(self.history[0])
        if RANK[card1] > RANK[card2]:
            return bet
        return -bet
