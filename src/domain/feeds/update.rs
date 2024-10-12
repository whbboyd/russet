//! # Design for update scheduling
//!
//! Schedule updates for each feed. Time-to-next-update should be stored on the
//! feed, and derived from some function of the following variables:
//!
//! * Fixed minimum and maximum update intervals
//! * Historical frequency of updates
//!		* For example, a feed which updates multiple times per day should be
//!			checked more often than one which updates weekly
//! * Historical schedule of updates
//!		* For example, if a feed updates daily at 04:17 UTC, we should strive to
//!			check that feed shortly thereafter. OTOH, a feed which updates
//!			irregularly should not cause weird update frequencies.
//! * The feed's own signals about update frequency: cache control headers,
//!		HTTP 429, etc.
//!
//! To do this, we add a table `feed_checks`:
//!
//! * `id` - the `fetch_index` currently stored in `metadata`
//! * `feed_id` - the feed which was checked
//! * `check_time` - the timestamp at which the check was performed
//! * `next-check-time` - the timestamp at which the next check should be
//!		performed
//! * `etag` - the etag, if any, 
//!
//! Then, at application start, schedule a check at `next-check-time` (or
//! immediately, if that time is in the past) for each feed. At every check, use
//! the status of that check (number of updates found, probably) to revise the
//! update metadata for that feed, then using that update metadata, schedule the
//! next check for the feed.
//!
//! ## Next check time prediction
//!
//! We're operating on the assumption that **feeds will either be very
//! periodic**, with updates at regular intervals; or **they will be essentially
//! random**, with the only relevant parameter the density of entries in time. We
//! wish to *distinguish which case* we're looking at, and then compute the
//! appropriate parameters (period and basetime for periodic feeds, density for
//! aperiodic ones) for that case.
//!
//! ### Initial design
//!
//! This design has not been proven yet, so may ultimately not be practicable
//! (in particular, it's not clear that it will work with the relatively small
//! volume of data we'll be working with).
//!
//! Bound entry timestamps by the check that returned them. Entry timestamps are
//! often unreliable, so: every entry time is bounded above by the time of the
//! check which added it (no time travel) and below by the time of the previous
//! check (which did not include it; no retroactive entries). Entries returned
//! by the initial sync with a feed should be excluded from this analysis, as
//! there is no way to validate their times.
//!
//! Take a DFT of the entry times for the feed. (This probably requires
//! "sampling" the discrete entry events? Or may not work at all for this sort
//! of data.) This gives us frequency components for the feed.
//!
//! If one frequency (or possibly multiple frequencies? FT should let use tease
//! out overlapping schedules…) exceeds a certain threshold, the feed is
//! **periodic** and that frequency defines the period. Figure out a base time
//! somehow (if elements in the DFT can be related back to samples in the input,
//! that would obviously be ideal; otherwise, do something dumb and heuristic);
//! then the next predicted check time is the minimum `base + n * period +
//! epsilon` greater than the time of this check. (We may want to compute an 
//! epsilon based on how "tight" the frequency is.)
//!
//! If no frequncy exceeds the threshold, the feed is **aperiodic**, and the
//! entry density is the duration beteen the first entry and the current check
//! time. The next predicted check time is `now + density + epsilon`.
//!
//! These analyses should be idempotent as specified. However, they may be
//! computationally costly. If so, we may want to store the parameters and
//! only update them on some checks.
//!
//! ## Feed signaling
//!
//! Feeds may signal their own preferences about how often they should be
//! queried, via e.g. the `Cache-Control` header or `HTTP 429` responses
//! (although note that these responses are very likely to be oblivious to the
//! actual update schedule of the feed). Russet modifies the predicted next
//! check time according to the feed's check time signals as follows:
//!
//! * If the response to the previous check inluded a `Cache-Control: max-age`
//!		header, and that `max-age` is less than the server's configured maximum
//!		check interval, bound the next check below by that `max-age`. (For
//!		example, if the predicted next check is in 3,600s but the `max-age` for
//!		this request specified 7,200s, the next check should be pushed out to
//!		7,200s. We choose to believe the server about how stable its content is.
//!		On the other hand, a `max-age` of 604,800s almost certainly indcates a
//!		server configuration divorced from its contents, so is ignored and the
//!		predicted 3,600s is used.
//! * If the response to the previous check was an HTTP 429 (Too Many Requests)
//!		with a `Retry-After` provided, bound the next check below by that
//!		`Retry-After`. If there is no `Retry-After`, bound the next check below
//!		by the default check interval. Russet should not normally trigger HTTP
//!		429 responses outside of pathological misconfigurations, as it will
//!		rarely make a request to a server more often than every few minutes, and
//!		more typically every few hours.
//!
//! ## Global check frequency bounds
//!
//! Finally, the application maintains its own minimum and maximum check
//! intervals it will allow. These are configurable. The defaults are a minimum
//! of 5min and a maximum of 24hr.
//!
//! Additionally, it's necessary to have a default interval when there is not
//! enough data to predict a next check. This is also configurable, with a
//! default of 1hr.
//!
//! ## Time creep
//!
//! Processing takes time. Network requests especially take a lot of time. In
//! order to prevent this from causing times to creep later, all time
//! computations are anchored against *the time at which this check was
//! scheduled*, not wall clock time. (If the check was executed immediately due
//! to application startup or addition of a new feed, the "scheduled" time is
//! considered to be the wall clock time at which the check was started.)
//!
//! ## Conditional requests
//!
//! If the previous check which did not make a conditional request was less than
//! the application maximum check interval ago, perform this check
//! conditionally:
//!
//! * If the previous check included an `ETag`, send `If-None-Match` with that
//!		ETag.
//! * Otherwise, send `If-Modified-Since` with the time of the previous check.
//!
//! If the previous check which did not make a conditional request was *more*
//! than the application maximum check interval ago, do not make a conditional
//! request, even if we have enough metadata to do so. This is a defense against
//! misbehaving server implemntations.
//!
//! ## Metadata needs
//!
//! Given all the above, we need, per feed, to store:
//!
//! * `feed_check_id` of most recent feed check
//! * `next_check_time` of next scheduled feed check
//! * `etag` if an `ETag` was included with the previous check response
//!
//! ## `FeedCheck`
//!
//! The `FeedCheck` data model struct has an `id`, which is the value formerly
//! known as `fetch_index`. It's an atomically-incremeting integer value. It's
//! mandatory on data returned from the persistence layer, but setting it is a
//! concern *of* the persistence layer. So, we have a design space to explore!
//!
//! ### `Optional` `id`
//!
//! Allow the domain layer to not provide `id`. Two obvious downsides:
//! * the `id` is actually mandatory on reads, so there will be pointless
//!		`unwrap`ping or `expect`ing it in the domain layer; and
//! * the domain layer could actually provide an `id` on write, which the
//!		persistence layer must then either ignore (potential logic error) or
//!		check at runtime (runtime error).
//!
//! ### Ignore `id` on write
//!
//! The domain layer sets a junk `id` on the struct it sends to the persistecne
//! layer, which the persistence layer ignores. The downside here is that we
//! have implicit behavior (`id` is ignored by the write path) that's not
//! reflected in the types.
//!
//! ### Optimistic locking
//!
//! The persistence layer provides a means to check the next appropriate `id`,
//! and fails writes which do not have an appropriate `id`. Domain layer retries
//! as necessary until the write succeeds.
//! * *Very* complicated for no clear reason.
//! * The "domain layer provided a garbage id" (logic error) and "domain layer
//!		lost the race" (normal operation in a concurren system) modes are
//!		indistinguishable.
//! * It's very unlikely, but possible, for a given domain layer coroutine to be
//!		starved of access to the write path.
//!
//! ### Duplicate data model (manual)
//!
//! Persistence layer accepts one model without `id` for writing, returns a
//! different one with `id` from reads. This results in code duplication that
//! must be kept in sync, and probably a bunch of otherwise-unnecessary
//! conversion code.
//!
//! ### Duplicate data model (codegen)
//!
//! Same as above, but one side or the other is automatically generated.
//! * Need to write code for codegen
//! * Behavior gets harder to follow
//! * Build gets more complicated and slower
//!
//! ### Give up on clock-independence
//!
//! Use a ULID for this ID, like is done for all the other IDs. These are
//! ostensibly ordered, *but* the order is according to the system clock at the
//! time they are generated. We're using an incrementing integer to *escape* the
//! non-monotonic tyrrany of the wall clock, and exchange meaningful durations
//! for causality-enforced monotonicity.
//!
//! ### Pessimistic locking
//!
//! Storage layer provides a means to lock the check table, pass the next
//! appropriate `id` to the domain layer, do some processing, take a result,
//! probably check the `id` (although strictly speaking `id` ordering is a
//! domain layer concern), write it, and unlock the table.
//! * This is also very complicated for no clear reason.
//! * Locks the check table for writes for the duration, limiting throughput
//!		(this shouldn't be a high-write-volume table, though)
//! * Allows a poorly-behaved domain layer to lock the table indefinitly (bad,
//!		probable deadlocks) or until a timeout (which just makes it back into
//!		optimistic locking).
//!
//! ----
//!
//! Let's start with "duplicate data model (manual)"—it introduces the least
//! incidental complexity. Keep an eye on the model complexity, though; if we
//! find ourselves adding a lot more fields to `FeedCheck`, or needing to do
//! this with other model types, it's probably time to reevaluate.
//!
