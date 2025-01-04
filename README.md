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
$ cargo run <path to data> 2>/dev/null
AOL
50'th percentile: 15.4s
68'th percentile: 16.4s
95'th percentile: 20.2s
99'th percentile: 39.4s
99.9'th percentile: 100.1s
99.99'th percentile: 174.8s
99.999'th percentile: 174.8s
samples=2921, minimum=9.6s, maximum=174.8s, mean=16.2s, stdev=5.8s

Apple
50'th percentile: 56.9s
68'th percentile: 75.0s
95'th percentile: 142.0s
99'th percentile: 180.6s
99.9'th percentile: 189.8s
99.99'th percentile: 223.7s
99.999'th percentile: 223.7s
samples=2919, minimum=13.1s, maximum=223.7s, mean=60.3s, stdev=42.8s

Gmail
50'th percentile: 17.3s
68'th percentile: 19.4s
95'th percentile: 57.8s
99'th percentile: 98.4s
99.9'th percentile: 167.4s
99.99'th percentile: 185.8s
99.999'th percentile: 185.8s
samples=2919, minimum=10.1s, maximum=185.8s, mean=22.5s, stdev=16.7s

Hotmail
50'th percentile: 19.9s
68'th percentile: 23.4s
95'th percentile: 99.4s
99'th percentile: 157.6s
99.9'th percentile: 180.9s
99.99'th percentile: 188.6s
99.999'th percentile: 188.6s
samples=2916, minimum=11.1s, maximum=188.6s, mean=31.1s, stdev=29.1s

Yahoo!
50'th percentile: 16.4s
68'th percentile: 17.8s
95'th percentile: 52.5s
99'th percentile: 106.9s
99.9'th percentile: 180.7s
99.99'th percentile: 180.8s
99.999'th percentile: 180.8s
samples=2919, minimum=10.1s, maximum=180.8s, mean=21.1s, stdev=17.6s
```

[HdrHistogram]: https://github.com/HdrHistogram/HdrHistogram
[hdrhistogram crate]: https://docs.rs/hdrhistogram/latest/hdrhistogram/
