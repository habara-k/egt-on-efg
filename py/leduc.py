import re
from typing import Optional

from game_builder import GameState, Player

Action = str
Obs = str
Card = str
RANK = {
    "2": 0,
    "3": 1,
    "4": 2,
    "5": 3,
    "6": 4,
    "7": 5,
    "8": 6,
    "9": 7,
    "T": 8,
    "J": 9,
    "Q": 10,
    "K": 11,
    "A": 12,
}


class LeducHoldem(GameState[Action, Obs]):
    def __init__(self, card_list: list[Card] = ["J", "Q", "K"]):
        self.card_list = card_list
        self.n_card = len(card_list)
        self.terminated: bool = False
        self.community_card: Optional[Card] = None
        self.round = "PreFlop"
        self.bet: list[int] = [1, 1]
        self.n_raise: int = 0
        self.history: list[Action] = []
        self.last_player: Optional[Player] = None

    def player(self) -> Optional[Player]:
        if self.terminated:
            return None
        if not self.history:
            return Player.C
        if self.round == "Flop" and self.community_card is None:
            return Player.C
        if self.last_player is Player.P1:
            return Player.P2
        return Player.P1

    def legal_actions(self) -> list[Action]:
        assert self.player() is not None
        if not self.history:
            return [
                f"{card1}{card2}"
                for card1 in self.card_list
                for card2 in self.card_list
            ]
        if self.round == "Flop" and self.community_card is None:
            card1, card2 = list(self.history[0])
            if card1 == card2:
                return list(filter(lambda card: card != card1, self.card_list))
            else:
                return self.card_list
        if self.history[-1] == "Raise":
            if self.n_raise == 2:
                return ["Fold", "Call"]
            return ["Fold", "Call", "Raise"]
        return ["Check", "Raise"]

    def step(self, action: Action):
        assert action in self.legal_actions()
        p = self.player()
        assert p is not None
        if len(action) == 1:
            self.community_card = action
        if action == "Raise":
            self.bet[p] = self.bet[Player.opponent(p)] + (
                2 if self.round == "PreFlop" else 4
            )
            self.n_raise += 1
        if action == "Check" and self.history[-1] == "Check":
            if self.round == "PreFlop":
                self.round = "Flop"
            else:
                self.terminated = True
        if action == "Call":
            self.bet[p] = self.bet[Player.opponent(p)]
            self.n_raise = 0
            if self.round == "PreFlop":
                self.round = "Flop"
            else:
                self.terminated = True
        if action == "Fold":
            self.terminated = True

        self.last_player = p
        self.history.append(action)

    def prob(self, action: Action) -> float:
        assert self.player() is Player.C

        if len(action) == 1:
            card1, card2 = list(self.history[0])
            if card1 == card2:
                assert action != card1
                return 1 / (self.n_card - 1)
            if action == card1 or action == card2:
                return 1 / (2 * (self.n_card - 1))
            return 1 / (self.n_card - 1)
        if len(action) == 2:
            card1, card2 = list(action)
            if card1 == card2:
                return 1 / (self.n_card * (2 * self.n_card - 1))
            else:
                return 2 / (self.n_card * (2 * self.n_card - 1))
        assert False

    def obs(self, player: Player) -> Obs:
        assert player is not Player.C
        hole_card = self.history[0][player]
        return str((hole_card, self.history[1:]))

    def payoff(self) -> float:
        assert self.player() is None
        if self.history[-1] == "Fold":
            if self.last_player is Player.P1:
                return -self.bet[0]
            return self.bet[1]

        assert self.bet[0] == self.bet[1]
        bet = self.bet[0]

        card1, card2 = list(self.history[0])
        if card1 == self.community_card:
            return bet
        if card2 == self.community_card:
            return -bet
        if card1 > card2:
            return bet
        if card1 < card2:
            return -bet
        return 0
