use std::collections::HashMap;
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
        let entry = entry.context("failed to walk the file tree")?;

        if let Some(file) =
            read_entry(&entry).context(format!("failed to read {}", entry.path().display()))?
        {
            data_files.push(file);
        }
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
                if let Some(pre) = previous_file
                    .contents
                    .iter()
                    .find(|pre_timings| pre_timings.name == cur.name)
                {
                    dedup(&cur.timings_millis, &pre.timings_millis).context(format!(
                        "failed to dedup {} data @ {} with previous data @ {}",
                        cur.name, file.timestamp, previous_file.timestamp
                    ))?
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

fn read_entry(entry: &DirEntry) -> Result<Option<DataFile>> {
    if !entry.file_type().is_file() {
        return Ok(None);
    }
    if entry
        .path()
        .extension()
        .map(|e| e != "json")
        .unwrap_or(true)
    {
        eprintln!("skipping non-JSON file {}", entry.path().display());
        return Ok(None);
    }

    let timestamp = timestamp_from_entry(&entry)
        .context("failed to determinate data file timestamp from its file name")?;

    let text = fs::read_to_string(entry.path()).context("failed to read file")?;
    if text.contains("Internal Server Error") {
        eprintln!(
            "skipping Internal Server Error response {}",
            entry.path().display()
        );
        return Ok(None);
    }
    let contents: Vec<Service> =
        serde_json::from_str(&text).context("failed to deserialize data")?;

    Ok(Some(DataFile {
        timestamp,
        contents,
    }))
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
// ```plain
//                      t0      t1
// A B C D E F G H I J K|       |
// | |     E F G H I J K|L M N O|
//  i      ^^^^^^^^^^^^^ ^^^^^^^
//           redundant     new
// ```
fn dedup(current: &[f64], previous: &[f64]) -> Result<Vec<f64>> {
    ensure!(current.len() == previous.len());
    let len = current.len();

    for overlap in (1..len).rev() {
        // Ignore the last value in `previous`, as it'd be partially duplicated.
        let c = &current[0..overlap];
        let p = &previous[(len - 1 - overlap)..(len - 1)];

        if c == p {
            if overlap < len - 1 {
                // Partial overlap found: remove it, the partially duplicated value with
                // `previous`, as well as the last value that'll eventually be partially
                // duplicated.
                return Ok(current[(overlap + 1)..(len - 1)].to_vec());
            } else {
                // Full overlap (except for the ignored partially duplicated last value): keep nothing.
                return Ok(vec![]);
            }
        }
    }

    // No overlap: remove the last value, as it'll be partially duplicated.
    Ok(current[0..(len - 1)].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        #[rustfmt::skip]
        let cur = [               4.0, 4.9, 6.0, 7.0, 8.0];
        let pre = [1.0, 2.0, 3.0, 4.0, 5.1];
        assert_eq!(dedup(&cur, &pre).unwrap(), vec![6.0, 7.0]);
    }

    #[test]
    fn full_overlap() {
        let cur = [1.0, 2.0, 3.0, 4.0, 4.9];
        let pre = [1.0, 2.0, 3.0, 4.0, 5.1];
        assert_eq!(dedup(&cur, &pre).unwrap(), Vec::<f64>::new());
    }

    #[test]
    fn min_overlap() {
        #[rustfmt::skip]
        let cur = [                    4.9, 6.0, 7.0, 8.0, 9.0];
        let pre = [1.0, 2.0, 3.0, 4.0, 5.1];
        assert_eq!(dedup(&cur, &pre).unwrap(), (vec![4.9, 6.0, 7.0, 8.0]));
    }

    #[test]
    fn confusing_overlap() {
        #[rustfmt::skip]
        let cur = [                         6.0, 7.0, 8.0, 9.0, 10.0];
        let pre = [1.0, 2.0, 3.0, 4.0, 5.1];
        assert_eq!(dedup(&cur, &pre).unwrap(), (vec![6.0, 7.0, 8.0, 9.0]));
    }

    #[test]
    fn no_possible_overlap() {
        #[rustfmt::skip]
        let cur = [                         6.0];
        let pre = [1.0];
        assert_eq!(dedup(&cur, &pre).unwrap(), Vec::<f64>::new());
    }
}
