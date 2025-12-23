# Development log

## 2025-12-22

_Actually_ I think I was _right_ about needing to overflow for insert! I'm
getting very vague and mixed messages from the resources online. Working it out
manually on a whiteboard was much more enlightening.

I'm going to use UC Berkeley's CS186 ("Introduction to Database Systems") as my
source of truth moving forward:

- Website: <https://cs186berkeley.net/notes/note4/>
- YouTube videos: <https://www.youtube.com/playlist?list=PLYp4IGUhNFmw8USiYMJvCUjZe79fvyYge>

## 2025-12-21

Actually I was wrong about needing to overflow for insert! I didn't read the
description of the insertion algorithm on Wikipedia ([link](https://en.wikipedia.org/wiki/B%2B_tree#Insertion))
carefully enough. It was late I guess.

## 2025-12-20

SQLite has been my main influence/inspiration so far, but I'm adding
[redb](https://github.com/cberner/redb) to the list:

> A simple, portable, high-performance, ACID, embedded key-value store. ... Data
> is stored in a collection of copy-on-write B-trees.

The copy-on-write aspect is particularly intriguing to me. At the moment (i.e.
naively) CoW feels elegant and powerful to me. Reminds me of [persistent data
structures](https://en.wikipedia.org/wiki/Persistent_data_structure) from
functional programming, and the [ZFS](https://en.wikipedia.org/wiki/ZFS)
filesystem, both of which I admire.

I finished implementing insert with overflow for leaf nodes, like you can insert
into an already full leaf and it will split itself and give you both the key to
put in the parent branch node and its new sibling leaf node.

I was struggling to implement the splitting because the `ArrayVec`s were
configured to only allow the "correct" number of keys and values to fill the
leaf. You'd need to consider where the incoming key and value fit in the
ordering without actually inserting it in its place in the list. Either by
constructing a new list, or iterating over the old one as you construct the new
leaves and constantly checking whether the incoming stuff should go here. That's
poorly explained, but it was challenging for me to understand, and ultimately I
abandoned that strategy.

After giving up and adding one more space in the `ArrayVec`s to "overflow" a
leaf, I was able to insert the incoming key and value into their expected
position and everything else fell into place so nicely. I don't know if the
literature and videos I was referencing were referring to "overflow" as a
pedagogical hand waving, or if that's literally how you're supposed to implement
it. I was thinking the former, but maybe I'm wrong. For now, I'll keep the extra
spot for overflow, because it makes for a vastly simpler implementation!

## 2025-12-19

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
