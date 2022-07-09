# Goldfisher 🎣

This tool aims to [goldfish](https://mtg.fandom.com/wiki/Goldfishing) the fastest possible wins with the given decks in non-interactive games of Magic: The Gathering. This data can be used to gather statistics on average winning turns with different versions of the decks, helping to gauge effects of deck building.

For now, the only supported deck is my build of [Premodern Pattern Rector](https://scryfall.com/@Cadiac/decks/79289c7a-f60c-4eff-809e-d83f86dd37c0), which contains a deterministic combo kill if the following condition is met:
1. Any repeatable sac outlet is available
2. You control a creature that isn't the sac outlet
3. You resolve a Pattern of Rector on the creature or you have Academy Rector on the battlefield

## Example game

```shell
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

## Installation

Follow [Rust](https://www.rust-lang.org/en-US/install.html) installation instructions.

## Running the tool

You can run develop builds of the tool with

```shell
$ cargo run
```
