from game_builder import build
from kuhn import KuhnPoker
from leduc import LeducHoldem


def main():
    with open("kuhn.json", "w") as f:
        f.write(build(KuhnPoker()))
    with open("leduc.json", "w") as f:
        f.write(build(LeducHoldem()))
    with open("leduc13.json", "w") as f:
        leduc13 = LeducHoldem(
            ["2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K", "A"]
        )
        f.write(build(leduc13))


if __name__ == "__main__":
    main()
