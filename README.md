# How do rust actors do?
This is a bit of a benchmark / example / evaluation of actors in Rust. To get things out of the way stright ahead, benchmarks are bad, don't take anything here and make a decision based on it without consideration.

The why you use something is more imporant then what you use in most of the cases. Something that was slower in these benchmarks might work better, more performant, and faster in your use case. Something that is faster here might work slower for you - you get the idea!

But then why make it? An exclent question! A few reasons. It gives a rought feeling of how different systems work in heavy message passing situations with a number of actors. It gives an idea how things scale inside of a given system based on number of actors, messages, paralellism and actor size. It also is nice to see the same algorithm implemented in differnt systems and learn a bit about all of them.

## But which one is the best?

Please go back up two paragraphs for this question - none, it's not about better.

## Why Erlang?

The Erlang implementation gives a good baseline. Especially since erlang is probably the most advanced implementation of actors at this point it makes sense to use it as a starting point. It also really shines at this so it gives a challenge! (we all like challenges don't we?)

# The algorithm

What do we test? We test how long it takes to pass a given number of messages around a ring of actors. Each actor that gets a message sends it to the next actor in the ring. A round trip is complete once the messages reaches the first actor again.
If we have a ring of 4 actors (A, B, C, D):

```
A -> B
^    |
|    v
D <- C
```

And we insert a message into `A` it will pass it to `B` which will pass it to `C` which will pass it to `D` which will then pass it to `A`. This is considered one round trip.

In the tests we would call that `4 processes, 1 message and a parallism of 1`.

If it makes a second trip around the ring, it would be `2 messages`. If we insert two messages instead of just one and they go their way around the ring we'd call it a `parallism of 2`.

You can think of it as a particle accelerator for messages. Sounds fun right?

## Performance

Please keep in mind that there are sure optimisations to be done, the first order of business is to get the cases running in a few different ways then to see how to tweek them - but as always tweaks and suggestions are more then welcome.

## references

The rust implementations do on purpose not use references as in any meaningful situation we would not shove the exact same message around over and over again but always have different messages.
