// Analyze the distribution of timestamps from scatter_triangle

use jeb_common::bi::scatter_triangle;

fn main() {
    // Simulate typical scenario:
    // - parent commit was 10 seconds ago
    // - current time is now (0)
    let min_timestamp = -10_i64;
    let target_timestamp = 0_i64;

    let mut accepted_author_past = 0;
    let mut accepted_author_future = 0;
    let mut accepted_committer_past = 0;
    let mut accepted_committer_future = 0;
    let mut rejected_by_author_constraint = 0;
    let mut rejected_by_committer_constraint = 0;

    // Check first 10000 indices from scatter_triangle
    for index in 0_u64..10000 {
        let (d_author, d_committer): (i32, i32) = scatter_triangle(index);
        let author_timestamp = target_timestamp + d_author as i64;
        let committer_timestamp = target_timestamp + d_committer as i64;

        // Current constraint: author_timestamp < min_timestamp
        if author_timestamp < min_timestamp {
            rejected_by_author_constraint += 1;
            continue;
        }

        // Alternative constraint: committer_timestamp < min_timestamp
        if committer_timestamp < min_timestamp {
            rejected_by_committer_constraint += 1;
        }

        // Track distribution
        if d_author < 0 {
            accepted_author_past += 1;
        } else {
            accepted_author_future += 1;
        }

        if d_committer < 0 {
            accepted_committer_past += 1;
        } else {
            accepted_committer_future += 1;
        }
    }

    let total_accepted = accepted_author_past + accepted_author_future;

    println!("Scenario: parent at t=-10, target at t=0");
    println!("\n=== Current Constraint (author_timestamp < min) ===");
    println!("Rejected: {}", rejected_by_author_constraint);
    println!("Accepted: {}", total_accepted);
    println!("\nAuthor timestamps:");
    println!("  Past:   {} ({:.1}%)", accepted_author_past,
             100.0 * accepted_author_past as f64 / total_accepted as f64);
    println!("  Future: {} ({:.1}%)", accepted_author_future,
             100.0 * accepted_author_future as f64 / total_accepted as f64);
    println!("\nCommitter timestamps:");
    println!("  Past:   {} ({:.1}%)", accepted_committer_past,
             100.0 * accepted_committer_past as f64 / total_accepted as f64);
    println!("  Future: {} ({:.1}%)", accepted_committer_future,
             100.0 * accepted_committer_future as f64 / total_accepted as f64);

    println!("\n=== Alternative Constraint (committer_timestamp < min) ===");
    println!("Would reject: {}", rejected_by_committer_constraint);
    println!("(This counts overlaps with current constraint)");

    // Show first few deltas to see the pattern
    println!("\n=== First 20 deltas ===");
    for index in 0_u64..20 {
        let (d_author, d_committer): (i32, i32) = scatter_triangle(index);
        let author_timestamp = target_timestamp + d_author as i64;
        let committer_timestamp = target_timestamp + d_committer as i64;
        let rejected_current = author_timestamp < min_timestamp;
        let rejected_alt = committer_timestamp < min_timestamp;
        println!("  [{:2}] ({:4}, {:4}) -> author={:3}, committer={:3} | current={} alt={}",
                 index, d_author, d_committer, author_timestamp, committer_timestamp,
                 if rejected_current { "REJECT" } else { "accept" },
                 if rejected_alt { "REJECT" } else { "accept" });
    }
}
