use leptos::prelude::*;

/// Fairness-enforcing preference sliders.
/// When a slider moves to value v, if too many slots have preference >= v,
/// automatically lower another slot's preference.
#[component]
pub fn WishSliders(
    slot_names: Vec<String>,
    wish: ReadSignal<Vec<i32>>,
    set_wish: WriteSignal<Vec<i32>>,
) -> impl IntoView {
    let n = slot_names.len();
    let max_val = if n > 0 { (n - 1) as i32 } else { 0 };

    view! {
        <table class="wish-table">
            <thead>
                <tr>
                    <th>"Slot"</th>
                    <th>"Your Wish"</th>
                </tr>
            </thead>
            <tbody>
                {slot_names
                    .into_iter()
                    .enumerate()
                    .map(|(i, name)| {
                        view! {
                            <tr>
                                <td>{name}</td>
                                <td>
                                    <div class="slider-cell">
                                        <span>"\u{1F4A9}"</span>
                                        <input
                                            type="range"
                                            min="0"
                                            max=max_val.to_string()
                                            step="1"
                                            prop:value=move || {
                                                let w = wish.get();
                                                if i < w.len() { max_val - w[i] } else { max_val }
                                            }
                                            on:input=move |ev| {
                                                let slider_val: i32 = crate::input_value(&ev).parse().unwrap_or(0);
                                                let new_value = max_val - slider_val;
                                                apply_fairness(i, new_value, &set_wish, n);
                                            }
                                        />
                                        <span>"\u{1F929}"</span>
                                    </div>
                                </td>
                            </tr>
                        }
                    })
                    .collect::<Vec<_>>()}
            </tbody>
        </table>
    }
}

fn apply_fairness(idx: usize, new_value: i32, set_wish: &WriteSignal<Vec<i32>>, n: usize) {
    set_wish.update(|wish| apply_fairness_vec(wish, idx, new_value, n));
}

/// Apply fairness constraint when setting slot `idx` to `new_value`.
/// Rule: when wishes are sorted ascending, wish[i] <= i for all i.
/// This means at most (n - v) slots can have preference >= v.
pub fn apply_fairness_vec(wish: &mut [i32], idx: usize, new_value: i32, n: usize) {
    let old_value = wish[idx];

    // Move incrementally toward new_value, enforcing constraint at each step
    if new_value > old_value {
        for v in (old_value + 1)..=new_value {
            wish[idx] = v;

            // Count how many slots have preference >= v
            let count = wish.iter().filter(|&&w| w >= v).count();

            if count > n - v as usize {
                // Too many high values — push one other slot down
                for (i, w) in wish.iter_mut().enumerate() {
                    if i != idx && *w == v {
                        *w = v - 1;
                        break;
                    }
                }
            }
        }
    }
    wish[idx] = new_value;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_fair(wish: &[i32]) -> bool {
        wish_shared::is_fair_wish(wish)
    }

    #[test]
    fn fairness_no_change_when_decreasing() {
        let mut wish = vec![0, 1, 2];
        apply_fairness_vec(&mut wish, 2, 0, 3);
        assert_eq!(wish, vec![0, 1, 0]);
        assert!(is_fair(&wish));
    }

    #[test]
    fn fairness_simple_increase() {
        let mut wish = vec![0, 0, 0];
        apply_fairness_vec(&mut wish, 0, 1, 3);
        assert_eq!(wish[0], 1);
        assert!(is_fair(&wish));
    }

    #[test]
    fn fairness_pushes_other_down() {
        // Start: [0, 1, 0] — move slot 0 to 1
        // Two slots would be >= 1 (slot 0 and slot 1), but only 2 allowed — ok for n=3
        // Actually n-v = 3-1 = 2, count=2, so 2 <= 2 — fine, no push
        let mut wish = vec![0, 1, 0];
        apply_fairness_vec(&mut wish, 2, 1, 3);
        // Now three slots >= 1? No: [0,1,1] — 2 slots >= 1, n-1=2, ok
        assert_eq!(wish, vec![0, 1, 1]);
        assert!(is_fair(&wish));
    }

    #[test]
    fn fairness_prevents_all_max() {
        // 3 slots, try to set all to 2 (max)
        // Only 1 slot can be >= 2 (n-v = 3-2 = 1)
        let mut wish = vec![0, 0, 0];
        apply_fairness_vec(&mut wish, 0, 2, 3);
        // slot 0 = 2, others stay 0 — fair
        assert_eq!(wish[0], 2);
        assert!(is_fair(&wish));

        apply_fairness_vec(&mut wish, 1, 2, 3);
        // Now we'd have two slots at 2, but only 1 allowed
        // The function should push one down
        assert_eq!(wish[1], 2);
        // One of the others should have been pushed to 1
        assert!(is_fair(&wish), "wish = {:?}", wish);
    }

    #[test]
    fn fairness_result_always_fair() {
        // Exhaustive test for 4 slots: try all transitions
        for start in 0..4i32 {
            for end in 0..4i32 {
                let mut wish = vec![0, 0, 0, 0];
                wish[0] = start;
                // First make sure starting state is fair
                if !is_fair(&wish) {
                    continue;
                }
                apply_fairness_vec(&mut wish, 0, end, 4);
                assert!(
                    is_fair(&wish),
                    "unfair after moving slot 0 from {start} to {end}: wish={wish:?}"
                );
            }
        }
    }

    #[test]
    fn fairness_exhaustive_all_slots_all_transitions() {
        // For every fair starting state (3 slots), try every transition on every slot
        let n = 3;
        let max_val = (n - 1) as i32;
        for a in 0..=max_val {
            for b in 0..=max_val {
                for c in 0..=max_val {
                    let start = vec![a, b, c];
                    if !is_fair(&start) {
                        continue;
                    }
                    for idx in 0..n {
                        for new_val in 0..=max_val {
                            let mut wish = start.clone();
                            apply_fairness_vec(&mut wish, idx, new_val, n);
                            assert!(
                                is_fair(&wish),
                                "unfair: start={start:?} idx={idx} new_val={new_val} → {wish:?}"
                            );
                            assert_eq!(wish[idx], new_val, "target slot not set correctly");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fairness_decrease_preserves_others() {
        let mut wish = vec![0, 1, 2];
        apply_fairness_vec(&mut wish, 1, 0, 3);
        // Decreasing should never change other slots
        assert_eq!(wish[0], 0);
        assert_eq!(wish[1], 0);
        assert_eq!(wish[2], 2);
    }

    #[test]
    fn fairness_set_to_same_value_is_noop() {
        let mut wish = vec![0, 1, 2];
        apply_fairness_vec(&mut wish, 1, 1, 3);
        assert_eq!(wish, vec![0, 1, 2]);
    }

    #[test]
    fn fairness_two_slots() {
        let mut wish = vec![0, 0];
        apply_fairness_vec(&mut wish, 0, 1, 2);
        assert!(is_fair(&wish));
        assert_eq!(wish[0], 1);
        // Only 1 slot can be >= 1 (n-v = 2-1 = 1), so other must be pushed to 0
        assert_eq!(wish[1], 0);
    }
}
