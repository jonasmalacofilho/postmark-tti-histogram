use std::collections::HashMap;
use std::time::Duration;
use std::{env, fs};

use anyhow::{Context, Result};
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
    for file in data_files {
        for service in file.contents {
            let histogram = histograms
                .entry(service.name.to_string())
                .or_insert_with(|| Histogram::new(4).unwrap());

            // TODO: try to deduplicate data already present in the preceeding file. This might not
            // be possible if all the data points we get are averages...

            for v in service.timings_millis {
                histogram.record(v.ceil() as u64)?;
            }
        }
    }

    let mut service_names: Vec<_> = histograms.keys().collect();
    service_names.sort();

    for (service, histogram) in service_names.iter().zip(histograms.values()) {
        println!("{}:", service);
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
