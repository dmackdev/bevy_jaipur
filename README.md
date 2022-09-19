# Bevy Jaipur

## Overview

A clone of the two player card game Jaipur, with local multiplayer and AI opponent modes. Made to learn the Rust game engine [Bevy](https://bevyengine.org/). (v0.8.0).

You can play this game in the browser here: https://dmackdev.itch.io/bevy-jaipur.

## Run the game

```bash
cargo run
```

## Build the game for Web

First ensure you have `wasm-bindgen-cli` at version 0.2.83 installed:

```bash
cargo install wasm-bindgen-cli@0.2.83
```

Run the following script to build the Web distribution:

```bash
./build-wasm.sh
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

## AI Notes

The Bevy plugin [big-brain](https://github.com/zkat/big-brain) is used for the AI player. As a [Utility AI](https://en.wikipedia.org/wiki/Utility_system) implementation, it "scores" possible moves during its turn according to their perceived benefit, and "picks" the move based on the score, according to some defined criteria.

### Scoring

(The following notes are more of a reminder for myself, but may be of interest to the reader. Read on if you have played at least one game and wish to understand how the AI "thinks" :smile:)

The Jaipur AI player picks the move with the highest score above 0 (picking the first move it considers for multiple moves with equal highest score). See the [highest score picker](src/ai/picker/highest_score.rs). I have also added a [weighted picker](src/ai/picker/weighted.rs) to pick moves with a probability proportionate to their score, but not tried/tested this.

The AI scoring implementation does not consider the values of the remaining game tokens for scoring possible "Take" moves, under the assumption that selling a higher quantity of goods, and possibly acquiring a Bonus Token, is better than focussing on the number of Rupees it would acquire.

For each possible move type on its turn - take single good, take all camels, exchange goods, sell goods - the AI produces a single score for the "best" valid selection of cards of that move type, and the move type with the highest score will be "picked". In order to save the selection of cards for each move type between the `big-brain` `Scorers` and `Action` stages, each scorer writes the card selection entities to a corresponding `ScorerState` component. The actions for each move type simply "select" the cards from this state by inserting a `SelectedCard` component, and fire a `ConfirmTurn` event with containing a payload indicating the move type. In this way, the same system for handling move execution for human players that reacts to this event, is reused.

The scoring formulae follow - percentages are used for readability, but in reality this are mapped to `0..1`.

### Sell goods

Scoring for selling goods considers only the good in hand with the highest frequency, and is proportionate to the number of the good in the hand, according to the formula `num_good * 20 / 100`. There is also a 1.5x multiplier for high value goods. High value goods with a frequency of under 2 are not considered, as this would be an invalid sell.

| Number of good | Score |
| -------------- | ----- |
| 5 or more      | 100%  |
| 4              | 80%   |
| 3              | 60%   |
| 2              | 40%   |
| 1              | 20%   |

### Take single good

Scoring for taking a single good follows a similar formula for Sell goods above - it uses corresponding percentages for the number of goods that would be in hand _after_ taking that good. It calculates the possible sub-scores for taking each good from the market, and writes the highest one. It uses the same 1.5x score multiplier for high value goods. As an improvement, this could score _higher_ than the Sell scorer for the same number of goods in hand after the take, since that could yield a better return with Bonus Tokens rather than selling prematurely.

| Number of good in hand after taking that good | Score |
| --------------------------------------------- | ----- |
| 5 or more                                     | 100%  |
| 4                                             | 80%   |
| 3                                             | 60%   |
| 2                                             | 40%   |
| 1                                             | 20%   |

### Take all camels

A good time to take camels is when:

- the market is full of camels (best bang for your buck of executing this move type), and
- the current player has few goods cards (on the next turn the player could obtain a large number of goods by exchanging camels), and
- the opponent has a full hand (the opponent cannot take a single good, cannot exchange camels for goods, and _may_ have to "sacrifice" goods in an exchange).

This gives us a scoring formula like:

```
num_camels_in_market - num_goods_in_hand + num_goods_in_opponent_hand
```

In the best case scenario this would be: `5 - 0 + 7 = 12`, but `num_goods_in_opponent_hand` has too much influence - even with 0 camels this component takes up more than half of the score.

We add weightings on these variables to put more importance on `num_camels_in_market` and less on `num_goods_in_hand`:

```
num_camels_in_market^2 - 2 * (0.5 * num*goods_in_hand)^2 + num_goods_in_opponent_hand
```

With this formula, our best case scenario becomes `5^2 - 0 + 7 = 32`, which we (arbitrarily) map to be equal score of selling 4 goods at 80% by scaling it:

```
[num_camels_in_market^2 - 2 * (0.5 * num_goods_in_hand)^2 + num_goods_in_opponent_hand] * 0.8/32
```

If current player has a full goods hand, it would be a bad time to take all camels since they cannot exchange the camels for goods on their next turn. These weightings cause the `num_camels_in_market` and `num_goods_in_hand` components in the formula to roughly cancel out when each is maximised, and yields a low score overall, but not zero since this is still a viable move:

```
[5^2 - 2 * (0.5 * 7)^2 + 7] * 0.8/32 = [25 - 24.5 + 7] * 0.8/32 = 0.1875
```

### Exchange goods

This was difficult since there could be many possible combinations for this move type, but the resulting implementation seems to make the AI perform somewhat sensible exchanges.

The scorer orders the goods in the market according to how many of that good would end up in its hand if it took all of that good in the market, highest to lowest. It identifies the goods in its hand of which there are only one, that are not in the market - these are deemed to not be as "valuable" to keep since acquiring more goods of the same type gets you closer to getting Bonus Tokens from a sale. Camels take precedence for exchanging - this is their main use in the game as they cannot be sold. We find the number of camels permitted to exchange - this would add extra goods to your hand, and you cannot have more than 7 goods in your hand. We "zip" the camels and single goods in your hand with the ordered goods in the market - this puts precedence on goods in the market that would yield the highest number in your hand, and ensures we would not exceed the max good limit since the goods being exchanged from your hand are 1-1 exchanges with good from the market. For a viable move, this result must have at least two tuples of a good/camel from your hand with a market good, since you must take at least two goods from the market in an exchange. For a viable move, we score this according to the formula `(highest_count_after_take + 1) * 20) / 100`, where `highest_count_after_take` is the highest number of a good in your hand that would occur after taking all of that good from the market. This is proportionate to the "Take single good" scoring formula, but we add 1 to make this a higher score for the same number of goods in your hand after the take since we would be exchanging less valuable cards (camels and single goods in hand) for more valuable goods from the market.

When I first played the AI without this move type as a possible move at all, it beat me, so maybe it would be better without this implementation at all! :satisfied:.
