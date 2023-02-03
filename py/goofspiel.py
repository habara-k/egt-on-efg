import copy
from typing import Optional

from game import GameState, Player

Card = str
Action = Card
Obs = str


class Goofspiel(GameState[Action, Obs]):
    def __init__(self, cards: list[Card] = ["A", "2", "3", "4"]):
        self.CARDS = cards
        self.CARD_NUM = len(self.CARDS)
        self.RANK = {card: i for i, card in enumerate(self.CARDS)}
        self.PRIZE = {card: i + 1 for i, card in enumerate(self.CARDS)}

        self.history: list[str] = []
        self.prize: Optional[Card] = None
        self.deck = [copy.deepcopy(self.CARDS) for _ in range(3)]
        self.payoff_sum = 0
        self.p1_action: Optional[Action] = None

    def player(self) -> Optional[Player]:
        if len(self.deck[Player.C]) == 0:
            return None
        if self.prize is None:
            return Player.C
        if self.p1_action is None:
            return Player.P1
        return Player.P2

    def legal_actions(self) -> list[Action]:
        player = self.player()
        assert player is not None
        return self.deck[player]

    def step(self, action: Action):
        player = self.player()
        if player is Player.C:
            self.prize = action
            return
        if player is Player.P1:
            self.p1_action = action
            return

        assert self.prize is not None
        assert self.p1_action is not None
        self.deck[Player.C].remove(self.prize)
        self.deck[Player.P1].remove(self.p1_action)
        self.deck[Player.P2].remove(action)
        if self.RANK[self.p1_action] < self.RANK[action]:
            self.payoff_sum += self.PRIZE[self.prize]
        elif self.RANK[self.p1_action] > self.RANK[action]:
            self.payoff_sum -= self.PRIZE[self.prize]
        self.history.append(f"{self.prize}{self.p1_action}{action}")
        self.prize = None
        self.p1_action = None

    def prob(self, _action: Action) -> float:
        assert self.player() is Player.C
        return 1.0 / len(self.deck[Player.C])

    def obs(self) -> Obs:
        assert self.prize is not None
        return ",".join([*self.history, self.prize])

    def payoff(self) -> float:
        assert self.player() is None
        return self.payoff_sum
