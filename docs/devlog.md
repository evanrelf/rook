# Development log

## 2026-01-15

More about LMDB:

- I found [this blog post](https://blog.separateconcerns.com/2016-04-03-lmdb-format.html)
  helpful for understanding the different page types, how overflow is handled,
  etc.

- Worth calling out that writes alternate between the two root pages, at index 0
  and 1, but naively that would mess up readers: if a read is happening starting
  from the root at index 0, and multiple writes occur, that page would change
  out from under the reader.

  My understanding is this problem is solved in two parts:

  1. The reader first makes an in-memory copy of the root page, and uses that
     instead of the volatile on-disk version.

  2. While there are readers, re-use of pages on the freelist is suspended, and
     instead the database reverts to a less efficient but safe append-only mode.
     That means all the pages reachable from the readers' roots are immutable
     and will not change out from under them.

## 2026-01-14

Some thoughts since my last update:

- I'm starting to read parts of ["Database Internals" by Alex Petrov](https://www.databass.dev/)
  and it pointed me towards the [LMDB database](https://en.wikipedia.org/wiki/Lightning_Memory-Mapped_Database).
  The book's description of LMDB closely matches my current design thinking for
  Rook, and the linked [presentation slides](https://databass.dev/links/87)
  provide even more satisfying details.

- I should probably move away from the data-oriented / struct-of-arrays style
  of storing keys and values/pointers contiguously in my b-tree pages. Instead,
  I should follow what LMDB and Postgres do, and store a list of fixed-size
  intra-page pointers after the page header, pointing to entries in the
  following data section. This allows for arbitrary size entries, and helps with
  fragmentation (allows entries to be out of order and have gaps).

- I like how LMDB stores its freelist as just another b-tree, referenced by the
  root page (in my case, the uber page).

## 2025-12-26

Taking a break from working on this during the holidays, but I've still been
thinking about it and reading up on different data structures and databases and
stuff...

- [LSM tree](https://en.wikipedia.org/wiki/Log-structured_merge-tree)

  Seems oriented towards write performance above all else. Reads sound worse
  than B-trees so I'm not interested in this for my project. TigerBeetle uses
  a forest of LSM trees.

- [Bε-tree](https://www.usenix.org/system/files/login/articles/login_oct15_05_bender.pdf)

  Takes a B+ tree and adds buffers to internal nodes for messages (e.g. insert,
  delete, update). Improves write performance (appending to buffer is O(1)),
  reduces write amplification (fewer nodes touched for mutations), etc.

  The reduced write amplification interests me most because that makes a
  copy-on-write version more efficient. Fewer nodes touched for mutations means
  fewer nodes copied. I think I will convert my existing B+ tree into a Bε-tree.
  Later if I want a regular B+ tree, I can just set ε to 1 and the buffers go
  away.

  Same as [fractal tree](https://en.wikipedia.org/wiki/Fractal_tree_index) I
  think? If so, [hitchhiker tree](https://www.youtube.com/watch?v=jdn617M3-P4)
  builds on this to make a persistent data structure.

- [STBε-tree](https://www.usenix.org/system/files/atc20-conway.pdf)

  This seems like a cool improvement over the Bε-tree, but the added complexity
  is overkill for my project.

Ultimately I think the B+ tree is fine for now. I need to follow the "make it
work, make it right, make it fast" philosophy and move on to moving the tree
from memory to disk. I can come back later and make it fast(er) with the
Bε-tree.

## 2025-12-22

_Actually_ I think I was _right_ about needing to overflow for insert! I'm
getting very vague and mixed messages from the resources online. Working it out
manually on a whiteboard was much more enlightening.

I'm going to use UC Berkeley's CS186 ("Introduction to Database Systems") as my
source of truth moving forward:

- Website: <https://cs186berkeley.net/notes/note4/>
- YouTube videos: <https://www.youtube.com/playlist?list=PLYp4IGUhNFmw8USiYMJvCUjZe79fvyYge>

I've finally finished the initial in-memory `BTreeMap` implementation! That took
me longer than expected. Working through the algorithms by hand, and finding a
good reference and sticking to it, ended up being a good move. Before that I was
kind of flailing around with lots of lower quality, conflicting sources. Was
amusing and relieving to see CS186 agree with the "deletion without rebalancing"
paper. Delete went from being the scariest part I was fearing to a completely
trivial final step!

Now I need to choose what to work on next. I want to move from in-memory to
on-disk sooner rather than later. I'm not sure what the best path is. Perhaps I
start by changing the in-memory pointers to logical pointers (i.e. ID numbers).
That way I can start to play with the notion of looking up pages in storage
without actually touching disk or moving from arbitrary Rust types to bytes. I
will defer to tomorrow Evan; it's getting late.

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
