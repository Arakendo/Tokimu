# Doom Ordered Partitioned Relational Composition Evidence

| Field | Retained value |
| --- | --- |
| Review | AR-0030 |
| Study | Source-authorized relational contribution classification |
| Scope | Doom-private, headless synthetic evidence |
| Stable contract | none |
| Renderer vocabulary | none |
| Public screen-column semantics | none |

## Authorized Question

Can multiple finite source-authorized occurrences classify disjoint or
overlapping portions of one contribution in retained Doom order without
inventing a free-standing occluder or allowing later authority to reopen a
terminal result?

The model is deliberately monotonic:

```text
original candidate domain
    -> authority A classifies only its finite overlap
    -> authority B sees only the remaining eligible domain
    -> authority C sees only the remaining eligible domain
    -> unclaimed remainder is unresolved/fail-open
```

Candidate source support remains independent from authority support. A nearer
or later authority cannot authorize lazy-map geometry outside the candidate's
own source occurrence.

## Retained Result

Run:

```powershell
cargo test -q -p hello-doom-visibility-conformance --lib
cargo run -q -p hello-doom-visibility-conformance --bin ordered_partition_composition_report
```

Observed result:

```text
tests=141 passed
fixture=ordered-partition-composition
authorities=2
fragments=2
retained=1
rejected=1
unresolved=0
conserved=true
renderer-policy=none
screen-columns=none
stable-contract=none
```

The synthetic gate also proves:

- later authority can classify the remainder outside the first authority;
- reversing overlapping authority order changes the semantic result;
- terminal retained, rejected and unresolved fragments are not reopened;
- unsupported lazy-map excess remains unresolved/fail-open;
- cutout contributions do not become solid authority;
- equal-order overlapping solid authorities fail open rather than acquiring an
  ad hoc priority.

## E1M1 Gate

The six retained E1M1 replay rays were rerun through the ordered wall and plane
occurrence observers. The result is more selective than the global-shell
`LOOK` hits implied:

| Ray | Global-shell candidate | Ordered candidate result | Finite authority result |
| --- | --- | --- | --- |
| 1 | wall linedef 247 | source-protocol rejected | linedef 250, SEG 404, finite view interval |
| 2 | ceiling subsector 104 | retained as two narrow view intervals | linedef 252, SEGs 121/127, finite full-view intervals and opening |
| 3 | wall linedef 247 | source-protocol rejected | linedef 250, SEG 404, finite view interval |
| 4 | ceiling subsector 149 | source-protocol rejected | ceiling subsector 130, one finite view interval |
| 5 | ceiling subsector 104 | source-protocol rejected | linedef 252, SEGs 121/127, finite view intervals and opening |
| 6 | wall linedef 230 | source-protocol rejected | linedef 250, SEG 404, finite view interval |

This establishes that five visual defects are not unresolved relational-depth
questions. Doom's ordered source protocol has already made terminal rejection
decisions, while global-shell realization reintroduces those contributions.
Passing them to the relational composer would violate the experiment's
monotonicity rule by reopening finalized source decisions.

Ray 2 remains eligible but cannot yet be represented honestly by the current
composer. Its candidate is a plane instance with destination view intervals;
its authority is a wall SEG with SEG-local source parameter, horizontal view
interval and vertical opening. They have no demonstrated common source
parameter. A scalar `sky boundary occurs earlier` relation, nearest ray hit or
invented cross-parameter mapping is not accepted as a finite comparison domain.

GPU presentation and generic AABB/frustum filtering therefore remain
downstream and disabled. The next gate is either a bounded, source-grounded
plane-versus-SEG common-domain derivation or an explicit review finding that
these occurrence families require a different Doom-private composition model.

## Disposition

The bounded ordered model survives its synthetic falsifier. It remains
Doom-private experimental preparation. No Tokimu renderer or stable API
admission follows from this result.
