# Goldfisher 🎣

This tool aims to [goldfish](https://mtg.fandom.com/wiki/Goldfishing) the fastest possible wins with the given decks in non-interactive games of Magic: The Gathering. This data can be used to gather statistics on average winning turns with different versions of the decks, helping to gauge effects of deck building.

For now, the only supported deck is my build of [Premodern Pattern Rector](https://scryfall.com/@Cadiac/decks/79289c7a-f60c-4eff-809e-d83f86dd37c0), which contains a deterministic combo kill if the following condition is met:
1. Any repeatable sac outlet is available
2. You control a creature that isn't the sac outlet
3. You resolve a Pattern of Rector on the creature or you have Academy Rector on the battlefield

## Installation

Follow [Rust](https://www.rust-lang.org/en-US/install.html) installation instructions.

## Usage

You can run develop builds of the tool by

```console
$ cargo run
```

## Example game

```
$ ./goldfisher
[Turn 00][Hand]: Pattern of Rebirth, Volrath's Shapeshifter, Goblin Bombardment, Llanowar Wastes, Gemstone Mine, Birds of Paradise, Karmic Guide
[Turn 00][Action]: Keeping a hand of 7 cards.
========================================
[Turn 01][Library]: 53 cards remaining.
[Turn 01][Hand]: Pattern of Rebirth, Volrath's Shapeshifter, Goblin Bombardment, Llanowar Wastes, Gemstone Mine, Birds of Paradise, Karmic Guide
[Turn 01][Battlefield]: 
[Turn 01][Graveyard]: 
[Turn 01][Action]: Playing land: "Gemstone Mine"
[Turn 01][Action]: Casting spell "Birds of Paradise" with mana sources: Gemstone Mine
========================================
[Turn 02][Library]: 52 cards remaining.
[Turn 02][Hand]: Pattern of Rebirth, Volrath's Shapeshifter, Goblin Bombardment, Llanowar Wastes, Karmic Guide, Mesmeric Fiend
[Turn 02][Battlefield]: Gemstone Mine, Birds of Paradise
[Turn 02][Graveyard]: 
[Turn 02][Action]: Playing land: "Llanowar Wastes"
[Turn 02][Action]: Casting spell "Goblin Bombardment" with mana sources: Gemstone Mine, Llanowar Wastes
========================================
[Turn 03][Library]: 51 cards remaining.
[Turn 03][Hand]: Pattern of Rebirth, Volrath's Shapeshifter, Karmic Guide, Mesmeric Fiend, Reflecting Pool
[Turn 03][Battlefield]: Goblin Bombardment, Llanowar Wastes, Gemstone Mine, Birds of Paradise
[Turn 03][Graveyard]: 
[Turn 03][Action]: Playing land: "Reflecting Pool"
[Turn 03][Action]: Casting spell "Pattern of Rebirth" on target "Birds of Paradise" with mana sources: Llanowar Wastes, Gemstone Mine, Birds of Paradise, Reflecting Pool
========================================
 Won the game on turn 3!
========================================
[Turn 03][Library]: 51 cards remaining.
[Turn 03][Hand]: Volrath's Shapeshifter, Karmic Guide, Mesmeric Fiend
[Turn 03][Battlefield]: Pattern of Rebirth, Goblin Bombardment, Llanowar Wastes, Gemstone Mine, Birds of Paradise, Reflecting Pool
[Turn 03][Graveyard]: 
```

## Results

```
Wins per turn after 100000 games:
Turn 03: 18998 wins (19.0%).
Turn 04: 22650 wins (22.6%).
Turn 05: 16483 wins (16.5%).
Turn 06: 10776 wins (10.8%).
Turn 07: 7154 wins (7.2%).
Turn 08: 5338 wins (5.3%).
Turn 09: 4113 wins (4.1%).
Turn 10: 3102 wins (3.1%).
Turn 11: 2493 wins (2.5%).
Turn 12: 1833 wins (1.8%).
Turn 13: 1501 wins (1.5%).
Turn 14: 1204 wins (1.2%).
Turn 15: 953 wins (1.0%).
Turn 16: 761 wins (0.8%).
Turn 17: 566 wins (0.6%).
Turn 18: 465 wins (0.5%).
Turn 19: 392 wins (0.4%).
Turn 20: 284 wins (0.3%).
Turn 21: 246 wins (0.2%).
Turn 22: 178 wins (0.2%).
Turn 23: 125 wins (0.1%).
Turn 24: 95 wins (0.1%).
Turn 25: 89 wins (0.1%).
Turn 26: 65 wins (0.1%).
Turn 27: 42 wins (0.0%).
Turn 28: 24 wins (0.0%).
Turn 29: 21 wins (0.0%).
Turn 30: 14 wins (0.0%).
Turn 31: 11 wins (0.0%).
Turn 32: 2 wins (0.0%).
Turn 33: 2 wins (0.0%).
Turn 34: 7 wins (0.0%).
Turn 35: 4 wins (0.0%).
Turn 36: 1 wins (0.0%).
Turn 37: 2 wins (0.0%).
Turn 38: 1 wins (0.0%).
Turn 39: 1 wins (0.0%).
Turn 40: 1 wins (0.0%).
Turn 41: 1 wins (0.0%).
Turn 42: 1 wins (0.0%).
Turn 44: 1 wins (0.0%).
```
