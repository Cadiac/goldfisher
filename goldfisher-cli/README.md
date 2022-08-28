# Goldfisher CLI 🎣

This tool aims to [goldfish](https://mtg.fandom.com/wiki/Goldfishing) the fastest possible wins with the given decks in non-interactive games of Magic: The Gathering. This data can be used to gather statistics on average winning turns with different versions of the decks, helping to gauge effects of deck building.

For now, the supported decks are:

[Premodern Pattern Combo](https://scryfall.com/@Cadiac/decks/79289c7a-f60c-4eff-809e-d83f86dd37c0), which contains a deterministic combo kill if the following condition is met:
1. Any repeatable sac outlet is available
2. You control a creature that isn't the sac outlet
3. You resolve a Pattern of Rebirth on the creature or you have Academy Rector on the battlefield

[Premodern Aluren](https://scryfall.com/@Cadiac/decks/a4aa0d03-bc15-44eb-bf4e-cb95d4dc8cbc), which goes off with Aluren by repeteatedly playing Cavern Harpy and bouncing creatures back to hand.

See [goldfisher](https://github.com/Cadiac/goldfisher/) library for the simple game engine implementation.

## Installation

Follow [Rust](https://www.rust-lang.org/en-US/install.html) installation instructions.

## Usage

```console
USAGE:
    goldfisher-cli [OPTIONS] --strategy <STRATEGY>

OPTIONS:
    -d, --decklist <DECKLIST>    Path to custom decklist file
    -g, --games <GAMES>          Number of games to simulate [default: 100]
    -h, --help                   Print help information
    -s, --strategy <STRATEGY>    The name of the deck strategy to use [possible values:
                                 pattern-combo, aluren]
    -v, --verbose                Print game actions debug output (slow)
    -V, --version                Print version information

```

You can run debug builds of the tool with cargo:

```console
$ cargo run -- --strategy pattern-combo --games 10 -v
```

For faster simulations adjust the `simulated_games` variable in `main.rs` and run executable as release build:

```console
$ cargo run --release --strategy pattern-combo --games 100000
```

To use your own custom decklist provide a path to the file:

```console
$ cargo run --release --strategy pattern-combo --games 100000 -d ./path/to/your/decklist.txt
```

## Example game

```console
goldfisher-cli --strategy pattern-combo --games 1 -v
[DEBUG] Deck size: 60
[DEBUG] ====================[ START OF GAME ]=======================
[DEBUG] [Turn 00][Action]: Drew card: "Pattern of Rebirth", 59 cards remaining.
[DEBUG] [Turn 00][Action]: Drew card: "Swamp", 58 cards remaining.
[DEBUG] [Turn 00][Action]: Drew card: "City of Brass", 57 cards remaining.
[DEBUG] [Turn 00][Action]: Drew card: "Goblin Bombardment", 56 cards remaining.
[DEBUG] [Turn 00][Action]: Drew card: "City of Brass", 55 cards remaining.
[DEBUG] [Turn 00][Action]: Drew card: "Phyrexian Ghoul", 54 cards remaining.
[DEBUG] [Turn 00][Action]: Drew card: "Reflecting Pool", 53 cards remaining.
[DEBUG] [Turn 00][Hand]: Phyrexian Ghoul, Pattern of Rebirth, Goblin Bombardment, Reflecting Pool, Swamp, City of Brass, City of Brass
[DEBUG] [Turn 00][Action]: Keeping a hand of 7 cards.
[DEBUG] ======================[ TURN 01 ]===========================
[DEBUG] [Turn 01][Game]: Life total: 20, Damage dealt: 0
[DEBUG] [Turn 01][Library]: 53 cards remaining.
[DEBUG] [Turn 01][Hand]: Phyrexian Ghoul, Pattern of Rebirth, Goblin Bombardment, Reflecting Pool, Swamp, City of Brass, City of Brass
[DEBUG] [Turn 01][Battlefield]: 
[DEBUG] [Turn 01][Graveyard]: 
[DEBUG] [Turn 01][Action]: Playing land: "City of Brass"
[DEBUG] ======================[ TURN 02 ]===========================
[DEBUG] [Turn 02][Action]: Drew card: "Carrion Feeder", 52 cards remaining.
[DEBUG] [Turn 02][Game]: Life total: 20, Damage dealt: 0
[DEBUG] [Turn 02][Library]: 52 cards remaining.
[DEBUG] [Turn 02][Hand]: Phyrexian Ghoul, Carrion Feeder, Pattern of Rebirth, Goblin Bombardment, Reflecting Pool, Swamp, City of Brass
[DEBUG] [Turn 02][Battlefield]: City of Brass
[DEBUG] [Turn 02][Graveyard]: 
[DEBUG] [Turn 02][Action]: Playing land: "City of Brass"
[DEBUG] [Turn 02][Action]: Casting card: "Carrion Feeder" with mana sources: "City of Brass"
[DEBUG] ======================[ TURN 03 ]===========================
[DEBUG] [Turn 03][Action]: Drew card: "Mesmeric Fiend", 51 cards remaining.
[DEBUG] [Turn 03][Game]: Life total: 20, Damage dealt: 0
[DEBUG] [Turn 03][Library]: 51 cards remaining.
[DEBUG] [Turn 03][Hand]: Phyrexian Ghoul, Mesmeric Fiend, Pattern of Rebirth, Goblin Bombardment, Reflecting Pool, Swamp
[DEBUG] [Turn 03][Battlefield]: Carrion Feeder, City of Brass, City of Brass
[DEBUG] [Turn 03][Graveyard]: 
[DEBUG] [Turn 03][Action]: Playing land: "Reflecting Pool"
[DEBUG] [Turn 03][Action]: Casting card: "Goblin Bombardment" with mana sources: "Reflecting Pool", "City of Brass"
[DEBUG] ======================[ TURN 04 ]===========================
[DEBUG] [Turn 04][Action]: Drew card: "Volrath's Shapeshifter", 50 cards remaining.
[DEBUG] [Turn 04][Game]: Life total: 20, Damage dealt: 0
[DEBUG] [Turn 04][Library]: 50 cards remaining.
[DEBUG] [Turn 04][Hand]: Phyrexian Ghoul, Volrath's Shapeshifter, Mesmeric Fiend, Pattern of Rebirth, Swamp
[DEBUG] [Turn 04][Battlefield]: Carrion Feeder, Goblin Bombardment, Reflecting Pool, City of Brass, City of Brass
[DEBUG] [Turn 04][Graveyard]: 
[DEBUG] [Turn 04][Action]: Playing land: "Swamp"
[DEBUG] [Turn 04][Action]: Casting card: "Pattern of Rebirth" on target "Carrion Feeder" with mana sources: "Reflecting Pool", "Swamp", "City of Brass", "City of Brass"
[DEBUG] =====================[ END OF GAME ]========================
[DEBUG]                       Win on turn 4!
[DEBUG] ============================================================
[DEBUG] [Turn 04][Game]: Life total: 20, Damage dealt: 0
[DEBUG] [Turn 04][Library]: 50 cards remaining.
[DEBUG] [Turn 04][Hand]: Phyrexian Ghoul, Volrath's Shapeshifter, Mesmeric Fiend
[DEBUG] [Turn 04][Battlefield]: Carrion Feeder, Pattern of Rebirth, Goblin Bombardment, Reflecting Pool, Swamp, City of Brass, City of Brass
[DEBUG] [Turn 04][Graveyard]: 
[INFO ] =======================[ RESULTS ]==========================
[INFO ]                    Average turn: 4.00
[INFO ]                Wins per turn after 1 games:
[INFO ] ============================================================
[INFO ] Turn 04: 1 wins (100.0%) - cumulative 100.0%
```

## Example results

```
goldfisher-cli --strategy pattern-combo --games 100000
[INFO ] =======================[ RESULTS ]==========================
[INFO ]                    Average turn: 5.50
[INFO ]               Wins per turn after 100000 games:
[INFO ] ============================================================
[INFO ] Turn 03: 22840 wins (22.8%) - cumulative 22.8%
[INFO ] Turn 04: 21765 wins (21.8%) - cumulative 44.6%
[INFO ] Turn 05: 18052 wins (18.1%) - cumulative 62.7%
[INFO ] Turn 06: 11247 wins (11.2%) - cumulative 73.9%
[INFO ] Turn 07: 7513 wins (7.5%) - cumulative 81.4%
[INFO ] Turn 08: 5088 wins (5.1%) - cumulative 86.5%
[INFO ] Turn 09: 3509 wins (3.5%) - cumulative 90.0%
[INFO ] Turn 10: 2386 wins (2.4%) - cumulative 92.4%
[INFO ] Turn 11: 1751 wins (1.8%) - cumulative 94.2%
[INFO ] Turn 12: 1154 wins (1.2%) - cumulative 95.3%
[INFO ] Turn 13: 815 wins (0.8%) - cumulative 96.1%
[INFO ] Turn 14: 532 wins (0.5%) - cumulative 96.7%
[INFO ] Turn 15: 396 wins (0.4%) - cumulative 97.0%
[INFO ] Turn 16: 293 wins (0.3%) - cumulative 97.3%
[INFO ] Turn 17: 217 wins (0.2%) - cumulative 97.6%
[INFO ] Turn 18: 159 wins (0.2%) - cumulative 97.7%
[INFO ] Turn 19: 125 wins (0.1%) - cumulative 97.8%
[INFO ] Turn 20: 107 wins (0.1%) - cumulative 97.9%
[INFO ] Turn 21: 60 wins (0.1%) - cumulative 98.0%
[INFO ] Turn 22: 42 wins (0.0%) - cumulative 98.1%
[INFO ] Turn 23: 28 wins (0.0%) - cumulative 98.1%
[INFO ] Turn 24: 20 wins (0.0%) - cumulative 98.1%
[INFO ] Turn 25: 12 wins (0.0%) - cumulative 98.1%
[INFO ] Turn 26: 15 wins (0.0%) - cumulative 98.1%
[INFO ] Turn 27: 11 wins (0.0%) - cumulative 98.1%
[INFO ] Turn 28: 5 wins (0.0%) - cumulative 98.1%
[INFO ] Turn 29: 3 wins (0.0%) - cumulative 98.1%
[INFO ] Turn 30: 3 wins (0.0%) - cumulative 98.1%
[INFO ] Turn 31: 3 wins (0.0%) - cumulative 98.2%
[INFO ] Turn 32: 1 wins (0.0%) - cumulative 98.2%
[INFO ] Turn 01: 45 losses (0.0%) - cumulative 0.0%
[INFO ] Turn 02: 83 losses (0.1%) - cumulative 0.1%
[INFO ] Turn 03: 151 losses (0.2%) - cumulative 0.3%
[INFO ] Turn 04: 182 losses (0.2%) - cumulative 0.5%
[INFO ] Turn 05: 191 losses (0.2%) - cumulative 0.7%
[INFO ] Turn 06: 199 losses (0.2%) - cumulative 0.9%
[INFO ] Turn 07: 195 losses (0.2%) - cumulative 1.0%
[INFO ] Turn 08: 167 losses (0.2%) - cumulative 1.2%
[INFO ] Turn 09: 139 losses (0.1%) - cumulative 1.4%
[INFO ] Turn 10: 106 losses (0.1%) - cumulative 1.5%
[INFO ] Turn 11: 69 losses (0.1%) - cumulative 1.5%
[INFO ] Turn 12: 59 losses (0.1%) - cumulative 1.6%
[INFO ] Turn 13: 46 losses (0.0%) - cumulative 1.6%
[INFO ] Turn 14: 43 losses (0.0%) - cumulative 1.7%
[INFO ] Turn 15: 35 losses (0.0%) - cumulative 1.7%
[INFO ] Turn 16: 30 losses (0.0%) - cumulative 1.7%
[INFO ] Turn 17: 25 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 18: 20 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 19: 10 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 20: 12 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 21: 10 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 22: 11 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 23: 4 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 24: 9 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 25: 3 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 26: 1 losses (0.0%) - cumulative 1.8%
[INFO ] Turn 28: 3 losses (0.0%) - cumulative 1.8%
```