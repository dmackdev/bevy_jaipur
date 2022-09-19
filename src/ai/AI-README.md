# Jaipur AI

The following notes are more of a reminder for myself, but may be of interest to the reader. Read on if you have played at least one game and wish to understand how the AI "thinks"! :smile:

### Scoring

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

When I first played the AI before I had implemented this move type, it beat me, so maybe it would be better without this implementation at all! :satisfied:
