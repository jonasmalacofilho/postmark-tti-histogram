use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Div;
use std::time::Duration;
use std::{env, fs};

use anyhow::{ensure, Context, Result};
use hdrhistogram::Histogram;
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Deserialize)]
struct Service {
    name: String,
    #[serde(rename = "values")]
    timings_millis: Vec<f64>,
}

#[derive(Debug)]
struct DataFile {
    timestamp: u64,
    contents: Vec<Service>,
}

fn main() -> Result<()> {
    let mut data_files = vec![];
    for entry in env::args().skip(1).flat_map(WalkDir::new) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry
            .path()
            .extension()
            .map(|e| e != "json")
            .unwrap_or(true)
        {
            eprintln!("skipping non-JSON file: {}", entry.path().display());
            continue;
        }

        let timestamp = timestamp_from_entry(&entry)
            .context(entry.path().display().to_string())
            .context("failed to determinate data file timestamp from its file name")?;
        let contents: Vec<Service> = serde_json::from_slice(&fs::read(entry.path())?)?;

        data_files.push(DataFile {
            timestamp,
            contents,
        });
    }
    data_files.sort_by_key(|f| f.timestamp);

    let mut histograms: HashMap<String, Histogram<u64>> = Default::default();
    let mut previous_file: Option<DataFile> = None;

    for file in data_files {
        for cur in &file.contents {
            let histogram = histograms
                .entry(cur.name.to_string())
                .or_insert_with(|| Histogram::new(4).unwrap());

            let timings = if let Some(ref previous_file) = previous_file {
                let dt = file.timestamp - previous_file.timestamp;
                if let Some(pre) = previous_file
                    .contents
                    .iter()
                    .find(|pre_timings| pre_timings.name == cur.name)
                {
                    dedup(&cur.timings_millis, &pre.timings_millis, dt)?.0
                } else {
                    todo!("check earlier files for redundant data for this service")
                }
            } else {
                cur.timings_millis.clone()
            };

            eprintln!(
                "// {}: {}: {} non-redudant samples",
                file.timestamp,
                cur.name,
                timings.len()
            );

            for v in timings {
                histogram.record(v.ceil() as u64)?;
            }
        }
        previous_file = Some(file);
    }
    eprintln!();

    let mut service_names: Vec<_> = histograms.keys().collect();
    service_names.sort();

    for (service, histogram) in service_names
        .into_iter()
        .map(|service| (service, histograms.get(service).unwrap()))
    {
        println!("{}", service);
        for quant in ["50", "68", "95", "99", "99.9", "99.99", "99.999"] {
            let millis = histogram.value_at_quantile(quant.parse::<f64>()? / 1E2);
            println!(
                "{}'th percentile: {:.1?}",
                quant,
                Duration::from_millis(millis)
            );
        }
        println!(
            "samples={}, minimum={:.1?}, maximum={:.1?}, mean={:.1?}, stdev={:.1?}",
            histogram.len(),
            Duration::from_millis(histogram.min()),
            Duration::from_millis(histogram.max()),
            Duration::from_secs_f64(histogram.mean() / 1e3),
            Duration::from_secs_f64(histogram.stdev() / 1e3),
        );
        println!();
    }

    Ok(())
}

fn timestamp_from_entry(entry: &DirEntry) -> Result<u64> {
    fn find_in(entry: &DirEntry) -> Option<&str> {
        entry
            .file_name()
            .to_str()?
            .strip_prefix("postmark-tti-")?
            .split_once('-')
            .map(|(prefix, _)| prefix)
    }
    find_in(entry)
        .context("data file name follow the pattern `postmark-tti-<timestamp>-*.json`")?
        .parse()
        .context("failed to parse timestamp from data file name as u64")
}

// Dedup the data coming from consecutive collections.
//
// For practical reasons (like not knowing exactly what we're getting from Postmark), we want to do
// think about without assuming that equivalent data points from different collections have
// *exactly* the same values.
//
// In ideal circunstances it would still be pretty simple to do dedup the data, even with the above
// restriction. The diagram bellow shows a regular stream of data points at `i` intervals, with
// collections at `t0` and `t1 = t0 + N * i, N ∈ ℕ`, and it's trivial to see that we should keep
// the last N data points from the second collection.
//
// ```plain
//                      t0      t1
// A B C D E F G H I J K|       |
// | |     E F G H I J K|L M N O|
//  i      ^^^^^^^^^^^^^ ^^^^^^^
//           redundant     new
// ```
//
// However, life isn't quite so simple: we need to account that the real intervals at which new
// data becomes available or that we make our collection requests have variance from their nominal
// intervals.
//
// For simplicity, let's keep the stream of data points regular (as it really doesn't matter which
// end is drifting from its nominal interval), but consider that our second collection happens at
// some `t1`, but we only known `t1* ∈ [ta, tb]`, and `t1*` was measured by a different clock than
// the one used for `t0`, `ta` and `tb`. In other words, we also can't assume to know how many
// intervals `i` have passed.
//
// ```plain
//                             t1*?
//                      t0    ta  tb
// A B C D E F G H I J K|     |   |
// | |     E F G H I J K|L M N|O  |
//  i      E F G H I J K|L M N|O P|
//         ^^^^^^^^^^^^^ ^^^^^^^^^
//           redundant      new
// ```
//
// Now the problem has gotten way too complicated. But, wait, there's still hope. While the clocks
// are different, we can assume that their rates are equal (within an acceptable level of
// precision), and that the skew between them is smaller than the nominal publication and
// collection invervals (but not insignificant).
//
// With these simplifications, we can calculate a real estimate `n'` of how many intervals have
// passed: `n' = (t1* - t0) / i, n' ∈ ℝ`. And given a maximum skew `δ < i`, the integer estimate
// `n` lies in the interval `[floor(n'-1), floor(n'+1)]`.
//
// This leaves us with just three possible cases to consider: `n == floor(n') - 1`, `n ==
// floor(n')` and `n == floor(n') + 1`. And we can choose the case that minimizes the error from
// the deduplication. We can compute this error in several ways, the most trivial yet robust being
// the sum of squared residuals; but if the data really does match exactly, a simple equality
// comparison would work too.
//
// However, there's one final thing to consider: all data points we get from Postmark appear to be
// the result of some aggregation of the true measurements they make. And because of this the last
// data point from one collection will frequently appear with a different value in the subsequent
// collection. (The aggregation they use might be the average of all measurements in fixed
// 15-minute windows).
//
// We could simply ignore this, and let least squares try to to make the best of it... and it would
// work ok. But instead let's take the opportunity only retain the worst of the two different
// values.
//
// ```plain
//                      t0      t1
// A B C D E F G H I J X|       |
// | |     E F G H I J Y|L M N O|
//  i      ^^^^^^^^^^^ ^ ^^^^^^^
//          redundant  *  new
//                 undecided
// ```
//
// Therefore, we initially want to keep `m = n + 1` intervals. The cases to consider are: `m ==
// floor(n')`, `m == floor(n') + 1` and `m == floor(n') + 2` and, once again, we choose the case
// that minimizes the (sum of the squares of the) error from the deduplication.
//
// Next, we check the partially redundant data point and pick the worst value for it.
//
// Finally, we always remove the last data point from the current collection, since at this time it's not yet
// known whether its worst value will be, and reconsider it on the next call/collection.
fn dedup(current: &[f64], previous: &[f64], timestamp_delta: u64) -> Result<(Vec<f64>, f64)> {
    ensure!(current.len() == previous.len());
    let len = current.len();

    // Compute a standardized error (similar to standard deviations from statistics) from
    // deduping `a` and `b`. The result is comparable with the values in `a` and `b`.
    //
    // Since we now know that all redundant points (except the last one, but this function is not
    // called with it) have exactly the same value in both current and previous collections, we
    // could simplify this to a simple equality comparison. But let's leave the least squares in,
    // as it's more robust.
    fn compute_error_millis(a: &[f64], b: &[f64]) -> f64 {
        assert_eq!(a.len(), b.len());
        if a.is_empty() {
            return 0.0;
        }
        a.iter()
            .zip(b)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .div(a.len() as f64)
            .sqrt()
    }

    let m: usize = (timestamp_delta / INTERVAL_SECS).try_into()?;
    if m == 0 {
        return Ok((vec![], 0.0));
    }

    let (best_keep, error_millis) = (m..=m + 2)
        .map(|keep| {
            if keep < len {
                let error_millis =
                    compute_error_millis(&current[..len - keep], &previous[keep - 1..len - 1]);
                (keep, error_millis)
            } else {
                (keep, 0.0)
            }
        })
        .min_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(Ordering::Equal)
                .then(a.0.cmp(&b.0))
        })
        .expect("candidates iterator should have more than one item");

    ensure!(
        error_millis < 1e3,
        "redundant data should match perfectly or near perfectly"
    );

    let mut ret = current[len.saturating_sub(best_keep)..len - 1].to_vec();
    // Try to differentiate between partial and full overlaps, even in the fact of uncertainty.
    // When in doubt, compare against the mean value `m`.
    if best_keep < len || best_keep != m {
        ret[0] = ret[0].max(previous[len - 1]);
    } else {
        ret.insert(0, previous[len - 1]);
    }
    Ok((ret, error_millis))
}

const INTERVAL_SECS: u64 = 900;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        #[rustfmt::skip]
        let cur = [               4.0, 4.9, 6.0, 7.0, 8.0];
        let pre = [1.0, 2.0, 3.0, 4.0, 5.1];
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 3).unwrap(),
            (vec![5.1, 6.0, 7.0], 0.0)
        );

        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 3 - 1).unwrap(),
            (vec![5.1, 6.0, 7.0], 0.0)
        );
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 4 - 1).unwrap(),
            (vec![5.1, 6.0, 7.0], 0.0)
        );
    }

    #[test]
    fn undecided_is_worst_in_current() {
        #[rustfmt::skip]
        let cur = [               4.0, 5.9, 6.0, 7.0, 8.0];
        let pre = [1.0, 2.0, 3.0, 4.0, 5.1];
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 3).unwrap(),
            (vec![5.9, 6.0, 7.0], 0.0)
        );

        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 3 - 1).unwrap(),
            (vec![5.9, 6.0, 7.0], 0.0)
        );
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 4 - 1).unwrap(),
            (vec![5.9, 6.0, 7.0], 0.0)
        );
    }

    #[test]
    fn full_overlap() {
        let cur = [1.0, 2.0, 3.0, 4.0, 4.9];
        let pre = [1.0, 2.0, 3.0, 4.0, 5.1];
        assert_eq!(dedup(&cur, &pre, 0).unwrap(), (vec![], 0.0));
        assert_eq!(dedup(&cur, &pre, INTERVAL_SECS - 1).unwrap(), (vec![], 0.0));
    }

    #[test]
    fn min_overlap() {
        #[rustfmt::skip]
        let cur = [                    4.9, 6.0, 7.0, 8.0, 9.0];
        let pre = [1.0, 2.0, 3.0, 4.0, 5.1];
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 4).unwrap(),
            (vec![5.1, 6.0, 7.0, 8.0], 0.0)
        );

        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 4 - 1).unwrap(),
            (vec![5.1, 6.0, 7.0, 8.0], 0.0)
        );
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 5 - 1).unwrap(),
            (vec![5.1, 6.0, 7.0, 8.0], 0.0)
        );
    }

    #[test]
    fn confusing_overlap() {
        #[rustfmt::skip]
        let cur = [                         6.0, 7.0, 8.0, 9.0, 10.0];
        let pre = [1.0, 2.0, 3.0, 4.0, 5.1];
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 5).unwrap(),
            (vec![5.1, 6.0, 7.0, 8.0, 9.0], 0.0)
        );

        // It's the best we can do for now... but feel free to change/improve.
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 5 - 1).unwrap(),
            (vec![6.0, 7.0, 8.0, 9.0], 0.0)
        );
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 6 - 1).unwrap(),
            (vec![5.1, 6.0, 7.0, 8.0, 9.0], 0.0)
        );
    }

    #[test]
    fn no_possible_overlap() {
        #[rustfmt::skip]
        let cur = [                         6.0];
        let pre = [1.0];
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 5).unwrap(),
            (vec![1.0], 0.0)
        );

        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 5 - 1).unwrap(),
            (vec![1.0], 0.0)
        );
        assert_eq!(
            dedup(&cur, &pre, INTERVAL_SECS * 6 - 1).unwrap(),
            (vec![1.0], 0.0)
        );
    }
}
