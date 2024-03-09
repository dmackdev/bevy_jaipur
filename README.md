# Bevy Jaipur

## Overview

A clone of the two player card game Jaipur, with local multiplayer and AI opponent modes. Made to learn the Rust game engine [Bevy](https://bevyengine.org/). (v0.8.0).

You can play this game in the browser here: https://dmackdev.itch.io/bevy-jaipur.

## Run the game

```bash
cargo run
```

## Build the game for Web

First ensure you have `wasm-bindgen-cli` installed:

```bash
cargo install wasm-bindgen-cli
```

Run the following script to build the Web distribution:

```bash
./build_wasm.sh
```

This outputs the Web build artifacts to `dist`, which includes a zipped folder `dist.zip`. The game can be served locally at http://localhost:3000 via:

```bash
python3 -m http.server --directory dist 3000
```

## How to play

The game follows the rules of Jaipur exactly - see https://www.fgbradleys.com/rules/rules2/Jaipur-rules.pdf for a full explanation. The objective of the game is to acquire the highest number of "Rupees", designated by the values of the game tokens. A game token is awarded for each good sold from your hand during a turn.

On your turn, select a "move mode" from the RHS buttons - either "Take" or "Sell".

Click on the cards that you wish to use for executing your move. Selected cards appear with a yellow outline. Click a selected card again to deselect it.

#### Take:

- Take single good: select a single good from the market. Valid when you have less than 7 goods in your hand.
- Exchange goods: select 2 or more goods from the market, and an equal number of: goods and camels from your hand. Valid when the exchange would not cause you to exceed 7 goods in your hand.
- Take all camels: select all camels from the market.

#### Sell:

- Select 1 or more goods, all of the same type, from your hand. For high value goods - diamond, silver, gold - must have at least 2 as part of the sale. Players additionally acquire a corresponding "Bonus Token" for sales of 3 goods or more.

When cards are selected such that the move is valid, the "Confirm" button will turn green. Press it to execute your move, and play passes to the other player.

(Note there is no automatic clearing of selected cards during a turn when changing the "move mode" - you may have leftover selected cards which would cause an invalid move.)

As per the rules, the game ends after a turn when either: the game tokens for 3 types of goods are depleted, or the market cannot be fully refilled from the deck. The player with the highest number of camels at the end of the game is awarded a 5 Rupee bonus. The player with the highest number of Rupees wins.

## AI

The Bevy plugin [big-brain](https://github.com/zkat/big-brain) is used for the AI player. As a [Utility AI](https://en.wikipedia.org/wiki/Utility_system) implementation, it "scores" possible moves during its turn according to their perceived benefit, and "picks" the move based on the score, according to some defined criteria.

See [here](src/ai/AI-README.md) for notes on my Jaipur AI implementation. Note that as per the rules - a player need not reveal the number of camels they have, so the AI player does not show you all of theirs.
