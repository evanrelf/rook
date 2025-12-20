# Development log

## 2025-12-19

### RUM conjecture

Watching Dmitrii Dolgov's "Modern B-Tree techniques" talk at Strange Loop 2022
([link](https://www.youtube.com/watch?v=4ELJDEjDpqk)) and he mentions the "RUM
conjecture" ([link](http://daslab.seas.harvard.edu/rum-conjecture/)):

> The fundamental challenges ... when designing a new access method are how to
> minimize, i) read times (R), ii) update cost (U), and iii) memory (or storage)
> overhead (M). In this project we first conjecture that when optimizing the
> read-update-memory overheads, optimizing in any two areas negatively impacts
> the third.

## 2025-12-18

Started project. I don't know what this is yet but I'm writing an in-memory
copy-on-write B+ tree I guess.

I learned recently that most of the time when people talk about B-trees they're
actually referring to B+ trees.
