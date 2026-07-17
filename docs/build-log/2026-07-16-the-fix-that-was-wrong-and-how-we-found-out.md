# The fix that was wrong, and how we found out

**2026-07-16**

The census had left one open question: what to actually do about thousands of false positives from the entropy and phone detectors. Three options were on the table. Add an allowlist at the policy layer for known-safe shapes. Tighten the detector heuristics directly. Or just accept it and let Task T10's formal precision metric be the first place it gets measured properly.

Rather than guess, that question went to a second model to plan through, on purpose, before any code changed. It came back with a hybrid: fix the two dominant detectors now, keep T10 as the real formal gate later, and specifically talked out of building a policy-layer allowlist yet, because the frozen policy contract has no hook for "suppress this one specific finding," only for classifying whole artefacts. Building that properly would mean reopening a contract that's supposed to be frozen, and a regex-based allowlist is also, worth saying plainly, a plausible way for something to quietly suppress a real secret if it's ever scoped too loosely. Reasonable advice, and it matched an independent read of the same code.

The phone fix was straightforward once decided: dates written as `2026-07-16` have eight digits split three ways, exactly the shape a short local phone number has, and that ambiguity was the majority of the phone detector's false positives. Excluding the strict calendar-date shape and nothing broader fixed it in one pass.

The entropy fix is the part worth actually telling. The working theory going in was that Hekton's own run identifiers, things like `run-20260608-EG-012`, were the dominant noise, since they mix digits and letters in exactly the density the entropy heuristic was tuned to catch. A detector-level exclusion got built for that specific shape and shipped with reasonable confidence. Then it got measured, properly, by isolating the exact same real corpus before and after the change. It removed one finding. Out of eighteen hundred and forty-nine.

The actual dominant noise wasn't run identifiers at all. It was file paths, `scripts/gateway-run.sh`, `.hekton/risk-register.yaml`, and ordinary code identifiers written in snake_case or kebab-case, `requires_confirmation`, `local-coding-harness`. The entropy tokenizer treats slashes, dots, and underscores as part of a token rather than a boundary, so an entire path or identifier gets scored as one blob, and the mix of letters, case, and punctuation clears the threshold even though every piece of it is an ordinary word. The first fix had been solving a real problem that happened not to be the actual problem.

The correction was to split a candidate token on its own internal delimiters and exclude it only when every resulting piece is either purely alphabetic or a short run of digits, which catches paths and identifiers generically rather than guessing at one specific naming convention. Measured the same way, on the same fixed corpus: entropy false positives down ninety percent, phone down ninety-one percent, latency untouched.

The uncomfortable part is worth saying without smoothing it over. An unmeasured, plausible-sounding fix almost shipped, and it would have looked like a real fix in every way that doesn't involve actually checking. That's the entire argument for building the census in the first place, and it's a little on the nose that the first thing it caught was its own author's assumption.

Full record: [`docs/decisions.md`](../decisions.md#2026-07-16---fixed-the-entropyphone-false-positive-finding-hybrid-detector-patch-now-t10-stays-the-gate).
