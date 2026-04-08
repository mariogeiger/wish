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
                if parent_worker_by_committed_job[j] == -1 {
                    if min_slack_value_by_job[j] < min_slack_value {
                        min_slack_value = min_slack_value_by_job[j];
                        min_slack_worker = min_slack_worker_by_job[j];
                        min_slack_job = j;
                    }
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
                let mut parent_worker =
                    parent_worker_by_committed_job[committed_job as usize];
                loop {
                    let temp = match_job_by_worker[parent_worker as usize];
                    match_job_by_worker[parent_worker as usize] = committed_job;
                    match_worker_by_job[committed_job as usize] = parent_worker;
                    committed_job = temp;
                    if committed_job == -1 {
                        break;
                    }
                    parent_worker =
                        parent_worker_by_committed_job[committed_job as usize];
                }
                break;
            } else {
                let worker = match_worker_by_job[min_slack_job] as usize;
                committed_workers[worker] = true;
                for j in 0..dim {
                    if parent_worker_by_committed_job[j] == -1 {
                        let slack =
                            cost[worker][j] - label_by_worker[worker] - label_by_job[j];
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
            for _ in 0..vmin {
                row.push(c);
            }
            let effective_vmax = vmax.min(num_participants as u32);
            for _ in vmin..effective_vmax {
                row.push(x + c);
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
        for (j, &(_vmin, vmax)) in slots.iter().enumerate() {
            let effective_vmax = vmax.min(num_participants as u32) as i32;
            if remaining < effective_vmax {
                slot_idx = j;
                break;
            }
            remaining -= effective_vmax;
        }
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
        let wishes = vec![
            vec![0, 1],
            vec![0, 1],
            vec![0, 1],
            vec![0, 1],
        ];
        let slots = vec![(2, 2), (2, 2)];
        let cost = build_cost_matrix(&wishes, &slots, 4);
        let assignment = hungarian(&cost);
        let result = assignment_to_slots(&assignment, &slots, 4);
        let in_slot_0 = result.iter().filter(|&&s| s == 0).count();
        let in_slot_1 = result.iter().filter(|&&s| s == 1).count();
        assert_eq!(in_slot_0, 2);
        assert_eq!(in_slot_1, 2);
    }
}
