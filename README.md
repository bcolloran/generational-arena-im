# `generational-arena-im`

A safe arena allocator that allows deletion without suffering from [the ABA
problem](https://en.wikipedia.org/wiki/ABA_problem) by using generational type-safe
indices. Forked from https://gitlab.com/tekne/typed-generational-arena.

Immutable via Im.

