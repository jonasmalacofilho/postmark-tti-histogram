# Collect and analyze Postmark Time to Inbox

[Postmark] measures the time it takes an email from entering their pipeline to it being (reported
as) delivered to the user's inbox. This is reportedly done at 5-minute intervals and against a few
of the major email services.

A summary of the data from the last 24 hours is published on [tti.postmarkapp.com], and the data
shown there is fetched from the [tti.postmarkapp.com/api/1.0/tti] endpoint. This data has 15-minute
granularity.

I would like to look at this data over a longer time frame than just the last 24 hours. So let's
collect the data for a while and analyze it.

[Postmark]: https://postmarkapp.com/
[tti.postmarkapp.com]: https://tti.postmarkapp.com
[tti.postmarkapp.com/api/1.0/tti]: https://tti.postmarkapp.com/api/1.0/tti

## Collection

Done using a simple systemd timer, see [`./systemd`](./systemd). Don't forget to adjust variables
and paths.

## Analysis

Latency data like this is known to almost always be non-normal, and we also care more about the bad
cases than the good ones.

For now, the data is only thrown into an [HdrHistogram] (specifically, using the [hdrhistogram]
crate), and a few basic metrics are printed.

## Example

```
$ cargo run path/to/postmark-tti-data/2025 2>/dev/null
AOL
50'th percentile: 15.6s
68'th percentile: 16.8s
95'th percentile: 23.3s
99'th percentile: 60.5s
99.9'th percentile: 97.6s
99.99'th percentile: 180.6s
99.999'th percentile: 181.0s
samples=24207, minimum=7.3s, maximum=181.0s, mean=17.2s, stdev=8.3s

Apple
50'th percentile: 24.0s
68'th percentile: 61.9s
95'th percentile: 141.6s
99'th percentile: 180.6s
99.9'th percentile: 180.9s
99.99'th percentile: 194.6s
99.999'th percentile: 202.9s
samples=23691, minimum=9.5s, maximum=202.9s, mean=49.9s, stdev=43.2s

Gmail
50'th percentile: 17.0s
68'th percentile: 18.9s
95'th percentile: 41.7s
99'th percentile: 72.0s
99.9'th percentile: 124.5s
99.99'th percentile: 180.5s
99.999'th percentile: 181.0s
samples=24160, minimum=7.3s, maximum=181.0s, mean=20.2s, stdev=11.7s

Hotmail
50'th percentile: 19.5s
68'th percentile: 22.1s
95'th percentile: 63.3s
99'th percentile: 114.1s
99.9'th percentile: 180.7s
99.99'th percentile: 181.3s
99.999'th percentile: 185.8s
samples=23934, minimum=8.4s, maximum=185.8s, mean=25.8s, stdev=19.3s

Yahoo!
50'th percentile: 16.2s
68'th percentile: 17.6s
95'th percentile: 41.6s
99'th percentile: 74.6s
99.9'th percentile: 133.1s
99.99'th percentile: 180.9s
99.999'th percentile: 190.1s
samples=24030, minimum=6.4s, maximum=190.1s, mean=19.3s, stdev=12.4s
```

[HdrHistogram]: https://github.com/HdrHistogram/HdrHistogram
[hdrhistogram crate]: https://docs.rs/hdrhistogram/latest/hdrhistogram/
