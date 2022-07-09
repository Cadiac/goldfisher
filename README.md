# Goldfisher 🎣

This tool aims to [goldfish](https://mtg.fandom.com/wiki/Goldfishing) the fastest possible wins with the given decks in non-interactive games of Magic: The Gathering. This data can be used to gather statistics on average winning turns with different versions of the decks, helping to gauge effects of deck building.

For now, the only supported deck is my build of [Premodern Pattern Rector](https://scryfall.com/@Cadiac/decks/79289c7a-f60c-4eff-809e-d83f86dd37c0), which contains a deterministic combo kill if the following condition is met:
1. Any repeatable sac outlet is available
2. You control a creature that isn't the sac outlet
3. You resolve a Pattern of Rector on the creature or you have Academy Rector on the battlefield

## Installation

Follow [Rust](https://www.rust-lang.org/en-US/install.html) installation instructions.

## Usage

You can run the tool with

```console
$ cargo run
```

For fast longer simulations adjust the `simulated_games` variable in `main.rs` and run executable as release build:

```console
$ cargo run --release
```

## Example game

```
$ ./goldfisher
[DEBUG] ====================[ START OF GAME ]=======================
[DEBUG] [Turn 00][Hand]: Pattern of Rebirth, Phyrexian Tower, Llanowar Elves, Nantuko Husk, Worship, Nantuko Husk, Goblin Bombardment
[DEBUG] [Turn 00][Action]: Taking a mulligan number 1.
[DEBUG] [Turn 00][Hand]: Cabal Therapy, Llanowar Wastes, Academy Rector, Llanowar Elves, Karmic Guide, City of Brass, Llanowar Elves
[DEBUG] [Turn 00][Action]: Keeping a hand of 6 cards.
[DEBUG] [Turn 00][Action]: Putting 1 cards on bottom: Karmic Guide
[DEBUG] ======================[ TURN 01 ]===========================
[DEBUG] [Turn 01][Library]: 54 cards remaining.
[DEBUG] [Turn 01][Hand]: Cabal Therapy, Llanowar Wastes, Academy Rector, Llanowar Elves, City of Brass, Llanowar Elves
[DEBUG] [Turn 01][Battlefield]: 
[DEBUG] [Turn 01][Graveyard]: 
[DEBUG] [Turn 01][Action]: Playing land: "City of Brass"
[DEBUG] [Turn 01][Action]: Casting spell "Llanowar Elves" with mana sources: City of Brass
[DEBUG] ======================[ TURN 02 ]===========================
[DEBUG] [Turn 02][Library]: 53 cards remaining.
[DEBUG] [Turn 02][Hand]: Cabal Therapy, Llanowar Wastes, Academy Rector, Llanowar Elves, Goblin Bombardment
[DEBUG] [Turn 02][Battlefield]: Llanowar Elves, City of Brass
[DEBUG] [Turn 02][Graveyard]: 
[DEBUG] [Turn 02][Action]: Playing land: "Llanowar Wastes"
[DEBUG] [Turn 02][Action]: Casting spell "Llanowar Elves" with mana sources: Llanowar Elves
[DEBUG] [Turn 02][Action]: Casting spell "Goblin Bombardment" with mana sources: City of Brass, Llanowar Wastes
[DEBUG] ======================[ TURN 03 ]===========================
[DEBUG] [Turn 03][Library]: 52 cards remaining.
[DEBUG] [Turn 03][Hand]: Cabal Therapy, Academy Rector, Worship
[DEBUG] [Turn 03][Battlefield]: Llanowar Wastes, Llanowar Elves, City of Brass, Llanowar Elves, Goblin Bombardment
[DEBUG] [Turn 03][Graveyard]: 
[DEBUG] [Turn 03][Action]: Casting spell "Academy Rector" with mana sources: City of Brass, Llanowar Elves, Llanowar Elves, Llanowar Wastes
[DEBUG] =====================[ END OF GAME ]========================
[DEBUG]  Won the game on turn 3!
[DEBUG] ============================================================
[DEBUG] [Turn 03][Library]: 52 cards remaining.
[DEBUG] [Turn 03][Hand]: Cabal Therapy, Worship
[DEBUG] [Turn 03][Battlefield]: Llanowar Wastes, Academy Rector, Llanowar Elves, City of Brass, Llanowar Elves, Goblin Bombardment
[DEBUG] [Turn 03][Graveyard]: 
```

## Results

```
[INFO ] =======================[ RESULTS ]==========================
[INFO ] Wins per turn after 500000 games:
[INFO ] ============================================================
[INFO ] Turn 03: 94943 wins (19.0%) - cumulative 19.0%
[INFO ] Turn 04: 113710 wins (22.7%) - cumulative 41.7%
[INFO ] Turn 05: 82055 wins (16.4%) - cumulative 58.1%
[INFO ] Turn 06: 52399 wins (10.5%) - cumulative 68.6%
[INFO ] Turn 07: 36951 wins (7.4%) - cumulative 76.0%
[INFO ] Turn 08: 26877 wins (5.4%) - cumulative 81.4%
[INFO ] Turn 09: 20426 wins (4.1%) - cumulative 85.5%
[INFO ] Turn 10: 15683 wins (3.1%) - cumulative 88.6%
[INFO ] Turn 11: 12015 wins (2.4%) - cumulative 91.0%
[INFO ] Turn 12: 9260 wins (1.9%) - cumulative 92.9%
[INFO ] Turn 13: 7467 wins (1.5%) - cumulative 94.4%
[INFO ] Turn 14: 6078 wins (1.2%) - cumulative 95.6%
[INFO ] Turn 15: 4699 wins (0.9%) - cumulative 96.5%
[INFO ] Turn 16: 3789 wins (0.8%) - cumulative 97.3%
[INFO ] Turn 17: 3041 wins (0.6%) - cumulative 97.9%
[INFO ] Turn 18: 2366 wins (0.5%) - cumulative 98.4%
[INFO ] Turn 19: 1953 wins (0.4%) - cumulative 98.7%
[INFO ] Turn 20: 1539 wins (0.3%) - cumulative 99.1%
[INFO ] Turn 21: 1167 wins (0.2%) - cumulative 99.3%
[INFO ] Turn 22: 899 wins (0.2%) - cumulative 99.5%
[INFO ] Turn 23: 692 wins (0.1%) - cumulative 99.6%
[INFO ] Turn 24: 525 wins (0.1%) - cumulative 99.7%
[INFO ] Turn 25: 403 wins (0.1%) - cumulative 99.8%
[INFO ] Turn 26: 284 wins (0.1%) - cumulative 99.8%
[INFO ] Turn 27: 226 wins (0.0%) - cumulative 99.9%
[INFO ] Turn 28: 165 wins (0.0%) - cumulative 99.9%
[INFO ] Turn 29: 108 wins (0.0%) - cumulative 99.9%
[INFO ] Turn 30: 90 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 31: 63 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 32: 50 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 33: 30 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 34: 17 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 35: 9 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 36: 8 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 37: 3 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 38: 3 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 39: 2 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 40: 2 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 41: 2 wins (0.0%) - cumulative 100.0%
[INFO ] Turn 44: 1 wins (0.0%) - cumulative 100.0%
```
