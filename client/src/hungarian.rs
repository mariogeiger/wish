// Hungarian algorithm uses intricate index math; iterator rewrites obscure
// the algorithm and aren't worth it here.
#![allow(clippy::needless_range_loop)]

use crate::parse::{self, ParseError, ParsedParticipant};
use wish_shared::Slot;

/// Run the full assignment pipeline: permute, build cost matrix, hungarian, un-permute.
/// Returns slot indices per participant (same order as input).
pub fn compute_assignment(slots: &[Slot], participants: &[ParsedParticipant]) -> Vec<usize> {
    let slots_data: Vec<(u32, u32)> = slots.iter().map(|s| (s.vmin, s.vmax)).collect();
    let n = participants.len();

    let perm: Vec<usize> = {
        let mut p: Vec<usize> = (0..n).collect();
        for i in (1..p.len()).rev() {
            let j = (js_sys::Math::random() * (i + 1) as f64) as usize;
            p.swap(i, j);
        }
        p
    };

    let wishes: Vec<Vec<i32>> = participants.iter().map(|p| p.wish.clone()).collect();

    let mut permuted_wishes = vec![Vec::new(); n];
    for (i, &pi) in perm.iter().enumerate() {
        permuted_wishes[pi] = wishes[i].clone();
    }

    let cost = build_cost_matrix(&permuted_wishes, &slots_data, n);
    let assignment = hungarian(&cost);
    let slot_indices = assignment_to_slots(&assignment, &slots_data, n);

    let mut result = vec![0usize; n];
    for i in 0..n {
        result[i] = slot_indices[perm[i]];
    }
    result
}

/// Parse text, run assignment, and format results in one call.
pub fn compute_and_format(text: &str) -> Result<String, Vec<ParseError>> {
    let parsed = parse::parse(text);
    if !parsed.errors.is_empty() {
        return Err(parsed.errors);
    }
    let result = compute_assignment(&parsed.slots, &parsed.participants);
    let participants_for_results: Vec<(String, Vec<i32>)> = parsed
        .participants
        .iter()
        .map(|p| (p.mail.clone(), p.wish.clone()))
        .collect();
    Ok(parse::format_results(
        &parsed.slots,
        &participants_for_results,
        &result,
    ))
}

/// Hungarian algorithm for the assignment problem. O(n^3).
/// Port of Kevin L. Stern's Java implementation (via Gerard Meier's JS port).
///
/// Given a cost matrix (workers x jobs), returns an assignment vector where
/// result[worker] = assigned job, or -1 if unassigned.
pub fn hungarian(cost_matrix: &[Vec<f64>]) -> Vec<i32> {
    if cost_matrix.is_empty() || cost_matrix[0].is_empty() {
        return Vec::new();
    }
    let rows = cost_matrix.len();
    let cols = cost_matrix[0].len();
    let dim = rows.max(cols);

    // Pad to square matrix
    let mut cost = vec![vec![0.0f64; dim]; dim];
    for (w, row) in cost_matrix.iter().enumerate() {
        for (j, &v) in row.iter().enumerate() {
            cost[w][j] = v;
        }
    }

    // Reduce rows
    for w in 0..dim {
        let min = cost[w].iter().cloned().fold(f64::INFINITY, f64::min);
        for j in 0..dim {
            cost[w][j] -= min;
        }
    }
    // Reduce columns
    let mut col_min = vec![f64::INFINITY; dim];
    for w in 0..dim {
        for j in 0..dim {
            col_min[j] = col_min[j].min(cost[w][j]);
        }
    }
    for w in 0..dim {
        for j in 0..dim {
            cost[w][j] -= col_min[j];
        }
    }

    let mut label_by_worker = vec![0.0f64; dim];
    let mut label_by_job = vec![0.0f64; dim];

    // Compute initial feasible solution
    for j in 0..dim {
        label_by_job[j] = f64::INFINITY;
    }
    for w in 0..dim {
        for j in 0..dim {
            if cost[w][j] < label_by_job[j] {
                label_by_job[j] = cost[w][j];
            }
        }
    }

    let mut match_job_by_worker = vec![-1i32; dim];
    let mut match_worker_by_job = vec![-1i32; dim];

    // Greedy match
    for w in 0..dim {
        for j in 0..dim {
            if match_job_by_worker[w] == -1
                && match_worker_by_job[j] == -1
                && (cost[w][j] - label_by_worker[w] - label_by_job[j]).abs() < 1e-10
            {
                match_job_by_worker[w] = j as i32;
                match_worker_by_job[j] = w as i32;
            }
        }
    }

    let mut min_slack_worker_by_job = vec![0usize; dim];
    let mut min_slack_value_by_job = vec![0.0f64; dim];
    let mut committed_workers = vec![false; dim];
    let mut parent_worker_by_committed_job = vec![-1i32; dim];

    // Augment
    let mut w = 0;
    while w < dim {
        // Find unmatched worker
        while w < dim && match_job_by_worker[w] != -1 {
            w += 1;
        }
        if w >= dim {
            break;
        }

        // Initialize phase
        committed_workers.fill(false);
        parent_worker_by_committed_job.fill(-1);
        committed_workers[w] = true;
        for j in 0..dim {
            min_slack_value_by_job[j] = cost[w][j] - label_by_worker[w] - label_by_job[j];
            min_slack_worker_by_job[j] = w;
        }

        // Execute phase
        loop {
            let mut min_slack_value = f64::INFINITY;
            let mut min_slack_worker = 0;
            let mut min_slack_job = 0;

            for j in 0..dim {
                if parent_worker_by_committed_job[j] == -1
                    && min_slack_value_by_job[j] < min_slack_value
                {
                    min_slack_value = min_slack_value_by_job[j];
                    min_slack_worker = min_slack_worker_by_job[j];
                    min_slack_job = j;
                }
            }

            if min_slack_value > 1e-10 {
                // Update labeling
                for ww in 0..dim {
                    if committed_workers[ww] {
                        label_by_worker[ww] += min_slack_value;
                    }
                }
                for j in 0..dim {
                    if parent_worker_by_committed_job[j] != -1 {
                        label_by_job[j] -= min_slack_value;
                    } else {
                        min_slack_value_by_job[j] -= min_slack_value;
                    }
                }
            }

            parent_worker_by_committed_job[min_slack_job] = min_slack_worker as i32;

            if match_worker_by_job[min_slack_job] == -1 {
                // Augmenting path found
                let mut committed_job = min_slack_job as i32;
                let mut parent_worker = parent_worker_by_committed_job[committed_job as usize];
                loop {
                    let temp = match_job_by_worker[parent_worker as usize];
                    match_job_by_worker[parent_worker as usize] = committed_job;
                    match_worker_by_job[committed_job as usize] = parent_worker;
                    committed_job = temp;
                    if committed_job == -1 {
                        break;
                    }
                    parent_worker = parent_worker_by_committed_job[committed_job as usize];
                }
                break;
            } else {
                let worker = match_worker_by_job[min_slack_job] as usize;
                committed_workers[worker] = true;
                for j in 0..dim {
                    if parent_worker_by_committed_job[j] == -1 {
                        let slack = cost[worker][j] - label_by_worker[worker] - label_by_job[j];
                        if min_slack_value_by_job[j] > slack {
                            min_slack_value_by_job[j] = slack;
                            min_slack_worker_by_job[j] = worker;
                        }
                    }
                }
            }
        }

        w += 1;
    }

    // Extract result for original workers only
    let mut result = match_job_by_worker[..rows].to_vec();
    for r in &mut result {
        if *r >= cols as i32 {
            *r = -1;
        }
    }
    result
}

/// Build cost matrix from participants' wishes and slot constraints.
/// Returns the expanded cost matrix where each slot is replicated vmin..vmax times.
pub fn build_cost_matrix(
    wishes: &[Vec<i32>],
    slots: &[(u32, u32)], // (vmin, vmax) per slot
    num_participants: usize,
) -> Vec<Vec<f64>> {
    let n_slots = slots.len();
    let mut x = (n_slots * n_slots) as f64;
    for wish in wishes {
        for &w in wish {
            x = x.max((w * w) as f64);
        }
    }

    let mut cost = Vec::with_capacity(num_participants);
    for wish in wishes {
        let mut row = Vec::new();
        for (j, &(vmin, vmax)) in slots.iter().enumerate() {
            let c = (wish[j] * wish[j]) as f64;
            let effective_vmax = vmax.min(num_participants as u32);
            for k in 0..effective_vmax {
                row.push(if k < vmin { c } else { x + c });
            }
        }
        cost.push(row);
    }
    cost
}

/// Convert hungarian result back to slot indices.
pub fn assignment_to_slots(
    assignment: &[i32],
    slots: &[(u32, u32)],
    num_participants: usize,
) -> Vec<usize> {
    let mut result = Vec::with_capacity(assignment.len());
    for &a in assignment {
        let mut remaining = a;
        let mut slot_idx = slots.len().saturating_sub(1);
        let mut found = false;
        for (j, &(_vmin, vmax)) in slots.iter().enumerate() {
            let effective_vmax = vmax.min(num_participants as u32) as i32;
            if remaining < effective_vmax {
                slot_idx = j;
                found = true;
                break;
            }
            remaining -= effective_vmax;
        }
        debug_assert!(found, "assignment index {} out of range for slot layout", a);
        result.push(slot_idx);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── hungarian() core algorithm ─────────────────────────────────

    #[test]
    fn hungarian_4x4() {
        let cost = vec![
            vec![10.0, 25.0, 15.0, 20.0],
            vec![15.0, 30.0, 5.0, 15.0],
            vec![35.0, 20.0, 12.0, 24.0],
            vec![17.0, 25.0, 24.0, 20.0],
        ];
        let result = hungarian(&cost);
        assert_eq!(result, vec![0, 2, 1, 3]);
    }

    #[test]
    fn hungarian_2x3_more_jobs() {
        let cost = vec![vec![1.0, 2.0, 3.0], vec![6.0, 5.0, 4.0]];
        let result = hungarian(&cost);
        assert_eq!(result, vec![0, 2]);
    }

    #[test]
    fn hungarian_3x2_more_workers() {
        let cost = vec![
            vec![1.0, 2.0, 3.0],
            vec![6.0, 5.0, 4.0],
            vec![1.0, 1.0, 1.0],
        ];
        let result = hungarian(&cost);
        assert_eq!(result, vec![0, 2, 1]);
    }

    #[test]
    fn hungarian_empty() {
        let cost: Vec<Vec<f64>> = vec![];
        let result = hungarian(&cost);
        assert!(result.is_empty());
    }

    #[test]
    fn hungarian_1x1() {
        let result = hungarian(&[vec![42.0]]);
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn hungarian_2x2_uniform() {
        let cost = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let result = hungarian(&cost);
        // Any valid assignment is fine — both workers to different jobs
        assert_ne!(result[0], result[1]);
    }

    #[test]
    fn hungarian_prefers_diagonal() {
        // Diagonal is cheapest
        let cost = vec![
            vec![1.0, 100.0, 100.0],
            vec![100.0, 1.0, 100.0],
            vec![100.0, 100.0, 1.0],
        ];
        let result = hungarian(&cost);
        assert_eq!(result, vec![0, 1, 2]);
    }

    // ── build_cost_matrix ──────────────────────────────────────────

    #[test]
    fn cost_matrix_basic_shape() {
        let wishes = vec![vec![0, 1], vec![1, 0]];
        let slots = vec![(1, 2), (1, 2)]; // vmin=1, vmax=2 each
        let cost = build_cost_matrix(&wishes, &slots, 2);
        assert_eq!(cost.len(), 2); // 2 participants
        assert_eq!(cost[0].len(), 4); // 2 slots x vmax=2 each
    }

    #[test]
    fn cost_matrix_vmin_columns_cheaper() {
        // With 3 participants, effective_vmax = min(2, 3) = 2, so each slot gets 2 columns
        let wishes = vec![vec![0, 1], vec![1, 0], vec![0, 0]];
        let slots = vec![(1, 2), (1, 2)];
        let cost = build_cost_matrix(&wishes, &slots, 3);
        // Slot 0 for participant 0: vmin col = 0^2 = 0, vmax col = x+0
        // vmin columns should be cheaper than vmax columns
        assert!(cost[0][0] < cost[0][1]); // slot 0: vmin < vmax
        assert!(cost[0][2] < cost[0][3]); // slot 1: vmin < vmax
    }

    // ── assignment_to_slots ────────────────────────────────────────

    #[test]
    fn assignment_to_slots_basic() {
        // 2 slots each vmax=2 → columns: [s0_0, s0_1, s1_0, s1_1]
        let slots = vec![(1, 2), (1, 2)];
        assert_eq!(assignment_to_slots(&[0], &slots, 2), vec![0]); // col 0 → slot 0
        assert_eq!(assignment_to_slots(&[1], &slots, 2), vec![0]); // col 1 → slot 0
        assert_eq!(assignment_to_slots(&[2], &slots, 2), vec![1]); // col 2 → slot 1
        assert_eq!(assignment_to_slots(&[3], &slots, 2), vec![1]); // col 3 → slot 1
    }

    // ── full pipeline ──────────────────────────────────────────────

    #[test]
    fn full_pipeline_everyone_gets_first_choice() {
        // 3 participants, 3 slots, each prefers a different slot
        let wishes = vec![
            vec![0, 1, 2], // prefers slot 0
            vec![2, 0, 1], // prefers slot 1
            vec![1, 2, 0], // prefers slot 2
        ];
        let slots = vec![(1, 1), (1, 1), (1, 1)];
        let cost = build_cost_matrix(&wishes, &slots, 3);
        let assignment = hungarian(&cost);
        let result = assignment_to_slots(&assignment, &slots, 3);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn full_pipeline_respects_vmax() {
        // 4 participants, 2 slots, everyone wants slot 0
        // but slot 0 has vmax=2, so 2 must go to slot 1
        let wishes = vec![vec![0, 1], vec![0, 1], vec![0, 1], vec![0, 1]];
        let slots = vec![(2, 2), (2, 2)];
        let cost = build_cost_matrix(&wishes, &slots, 4);
        let assignment = hungarian(&cost);
        let result = assignment_to_slots(&assignment, &slots, 4);
        let in_slot_0 = result.iter().filter(|&&s| s == 0).count();
        let in_slot_1 = result.iter().filter(|&&s| s == 1).count();
        assert_eq!(in_slot_0, 2);
        assert_eq!(in_slot_1, 2);
    }

    // ── full pipeline with permutation (admin/offline flow) ────────

    /// Helper: run the exact same pipeline as admin.rs/offline.rs on_compute,
    /// but with a deterministic permutation instead of random.
    fn run_assignment_pipeline(
        wishes: &[Vec<i32>],
        slots: &[(u32, u32)],
        perm: &[usize],
    ) -> Vec<usize> {
        let n = wishes.len();

        // Permute wishes (same logic as admin.rs:213-216 / offline.rs:60-63)
        let mut permuted_wishes = vec![Vec::new(); n];
        for (i, &pi) in perm.iter().enumerate() {
            permuted_wishes[pi] = wishes[i].clone();
        }

        let cost = build_cost_matrix(&permuted_wishes, slots, n);
        let assignment = hungarian(&cost);
        let slot_indices = assignment_to_slots(&assignment, slots, n);

        // Un-permute (same logic as admin.rs:223-226 / offline.rs:69-72)
        let mut result = vec![0usize; n];
        for i in 0..n {
            result[i] = slot_indices[perm[i]];
        }
        result
    }

    #[test]
    fn pipeline_with_identity_permutation() {
        let wishes = vec![
            vec![0, 1, 2], // prefers slot 0
            vec![2, 0, 1], // prefers slot 1
            vec![1, 2, 0], // prefers slot 2
        ];
        let slots = vec![(1, 1), (1, 1), (1, 1)];
        let perm: Vec<usize> = (0..3).collect(); // identity
        let result = run_assignment_pipeline(&wishes, &slots, &perm);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn pipeline_with_reversed_permutation() {
        let wishes = vec![
            vec![0, 1, 2], // prefers slot 0
            vec![2, 0, 1], // prefers slot 1
            vec![1, 2, 0], // prefers slot 2
        ];
        let slots = vec![(1, 1), (1, 1), (1, 1)];
        let perm = vec![2, 1, 0]; // reversed
        let result = run_assignment_pipeline(&wishes, &slots, &perm);
        // Same optimal assignment regardless of permutation
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn pipeline_permutation_does_not_change_optimal_result() {
        // All permutations of 3 participants should yield the same result
        let wishes = vec![
            vec![0, 2], // prefers slot 0
            vec![2, 0], // prefers slot 1
        ];
        let slots = vec![(1, 1), (1, 1)];

        let perms = vec![vec![0, 1], vec![1, 0]];
        for perm in &perms {
            let result = run_assignment_pipeline(&wishes, &slots, perm);
            assert_eq!(result, vec![0, 1], "failed with perm {:?}", perm);
        }
    }

    #[test]
    fn pipeline_respects_vmin_with_permutation() {
        // 4 participants, 2 slots with vmin=2 each
        // Everyone prefers slot 0, but vmin forces 2 into each
        let wishes = vec![vec![0, 1], vec![0, 1], vec![0, 2], vec![0, 2]];
        let slots = vec![(2, 2), (2, 2)];
        let perm = vec![3, 1, 0, 2]; // arbitrary permutation
        let result = run_assignment_pipeline(&wishes, &slots, &perm);
        let in_slot_0 = result.iter().filter(|&&s| s == 0).count();
        let in_slot_1 = result.iter().filter(|&&s| s == 1).count();
        assert_eq!(in_slot_0, 2);
        assert_eq!(in_slot_1, 2);
    }

    #[test]
    fn pipeline_asymmetric_slots() {
        // 3 participants, slot 0 holds 1, slot 1 holds 2
        let wishes = vec![
            vec![0, 1], // prefers slot 0
            vec![1, 0], // prefers slot 1
            vec![1, 0], // prefers slot 1
        ];
        let slots = vec![(1, 1), (2, 2)];
        let perm = vec![2, 0, 1];
        let result = run_assignment_pipeline(&wishes, &slots, &perm);
        // Participant 0 gets slot 0 (their preference), 1 and 2 get slot 1
        assert_eq!(result[0], 0);
        assert_eq!(result.iter().filter(|&&s| s == 0).count(), 1);
        assert_eq!(result.iter().filter(|&&s| s == 1).count(), 2);
    }

    #[test]
    fn pipeline_single_participant() {
        let wishes = vec![vec![0]];
        let slots = vec![(1, 1)];
        let perm = vec![0];
        let result = run_assignment_pipeline(&wishes, &slots, &perm);
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn pipeline_all_same_preference() {
        // 3 participants all rank slots identically
        let wishes = vec![vec![0, 1, 2], vec![0, 1, 2], vec![0, 1, 2]];
        let slots = vec![(1, 1), (1, 1), (1, 1)];
        let perm = vec![1, 2, 0];
        let result = run_assignment_pipeline(&wishes, &slots, &perm);
        // Each slot must get exactly 1 person
        let mut counts = vec![0; 3];
        for &s in &result {
            counts[s] += 1;
        }
        assert_eq!(counts, vec![1, 1, 1]);
        // And at least one should get their first choice
        assert!(result.contains(&0));
    }

    // ── cost matrix edge cases ─────────────────────────────────────

    #[test]
    fn cost_matrix_single_slot() {
        let wishes = vec![vec![3], vec![1]];
        let slots = vec![(2, 2)];
        let cost = build_cost_matrix(&wishes, &slots, 2);
        assert_eq!(cost.len(), 2);
        assert_eq!(cost[0].len(), 2); // vmax=2, so 2 columns
        assert_eq!(cost[0][0], 9.0); // 3^2
        assert_eq!(cost[1][0], 1.0); // 1^2
    }

    #[test]
    fn cost_matrix_vmax_capped_by_participants() {
        // vmax=100 but only 2 participants — effective_vmax = 2
        let wishes = vec![vec![0], vec![1]];
        let slots = vec![(1, 100)];
        let cost = build_cost_matrix(&wishes, &slots, 2);
        assert_eq!(cost[0].len(), 2); // min(100, 2) = 2 columns
    }

    #[test]
    fn assignment_to_slots_with_vmin_vmax_spread() {
        // 3 slots: vmin=1 vmax=3 each → 3+3+3 = 9 columns
        let slots = vec![(1, 3), (1, 3), (1, 3)];
        // Column layout: [s0_0, s0_1, s0_2, s1_0, s1_1, s1_2, s2_0, s2_1, s2_2]
        assert_eq!(assignment_to_slots(&[0], &slots, 3), vec![0]); // col 0 → slot 0
        assert_eq!(assignment_to_slots(&[2], &slots, 3), vec![0]); // col 2 → slot 0
        assert_eq!(assignment_to_slots(&[3], &slots, 3), vec![1]); // col 3 → slot 1
        assert_eq!(assignment_to_slots(&[5], &slots, 3), vec![1]); // col 5 → slot 1
        assert_eq!(assignment_to_slots(&[6], &slots, 3), vec![2]); // col 6 → slot 2
        assert_eq!(assignment_to_slots(&[8], &slots, 3), vec![2]); // col 8 → slot 2
    }
}
