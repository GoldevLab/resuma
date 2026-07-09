use resuma::core::context::{RenderContext, RenderMode};
use resuma::core::{signal, use_computed, use_effect, with_context};

#[test]
fn chained_computed_cascade_does_not_panic() {
    let ctx = RenderContext::new(RenderMode::Ssr);
    with_context(ctx, || {
        let sig = signal(1i64);

        let s1 = sig.clone();
        let c1 = use_computed(move || s1.get() * 2);

        let c1r = c1;
        let c2 = use_computed(move || c1r.get() + 1);

        sig.set(10);

        assert_eq!(c2.peek(), 21, "c2 should be sig*2+1 = 21");
    });
}

#[test]
fn conditional_computed_tracks_active_branch_only() {
    let ctx = RenderContext::new(RenderMode::Ssr);
    with_context(ctx, || {
        let cond = signal(true);
        let a = signal(1_i32);
        let b = signal(10_i32);

        let cond_r = cond.clone();
        let a_r = a.clone();
        let b_r = b.clone();
        let c = use_computed(move || if cond_r.get() { a_r.get() } else { b_r.get() });

        assert_eq!(c.peek(), 1);

        cond.set(false);
        assert_eq!(c.peek(), 10, "should follow b after cond flips");

        b.set(20);
        assert_eq!(c.peek(), 20, "should react to b in else branch");

        a.set(999);
        assert_eq!(c.peek(), 20, "should ignore a when cond is false");
    });
}

#[test]
fn mutual_effect_cycle_breaks_without_panic_or_deadlock() {
    // Two effects that write the signal the other reads form an A→B→A cycle.
    // The `running_effects` guard must break re-entry deterministically instead
    // of deadlocking on the RefCell or recursing forever. (Panic-on-cycle only
    // triggers when RESUMA_DEV is set, which it is not in tests.)
    let ctx = RenderContext::new(RenderMode::Ssr);
    with_context(ctx, || {
        let a = signal(0_i32);
        let b = signal(0_i32);

        let a1 = a.clone();
        let b1 = b.clone();
        let _e1 = use_effect(move || {
            b1.set(a1.get() + 1);
        });

        let a2 = a.clone();
        let b2 = b.clone();
        let _e2 = use_effect(move || {
            a2.set(b2.get() + 1);
        });

        // Must return (no hang / stack overflow / RefCell double-borrow panic).
        a.set(10);

        // State remains readable and finite after the cycle is broken.
        assert!(a.peek() >= 10);
        assert!(b.peek() >= 1);
    });
}

#[test]
fn computed_runs_closure_once_on_init() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    let ctx = RenderContext::new(RenderMode::Ssr);
    let runs = Arc::new(AtomicU32::new(0));
    with_context(ctx, || {
        let runs = runs.clone();
        let _c = use_computed(move || {
            runs.fetch_add(1, Ordering::Relaxed);
            42_i32
        });
    });
    assert_eq!(
        runs.load(Ordering::Relaxed),
        1,
        "compute closure must run exactly once during init"
    );
}
