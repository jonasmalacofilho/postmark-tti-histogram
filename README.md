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
50'th percentile: 15.3s
68'th percentile: 16.3s
95'th percentile: 19.6s
99'th percentile: 28.1s
99.9'th percentile: 80.1s
99.99'th percentile: 127.3s
99.999'th percentile: 174.8s
samples=15091, minimum=9.1s, maximum=174.8s, mean=15.9s, stdev=5.3s

Apple
50'th percentile: 60.6s
68'th percentile: 79.3s
95'th percentile: 171.6s
99'th percentile: 180.7s
99.9'th percentile: 181.0s
99.99'th percentile: 194.6s
99.999'th percentile: 223.7s
samples=14989, minimum=11.4s, maximum=223.7s, mean=65.8s, stdev=47.7s

Gmail
50'th percentile: 17.5s
68'th percentile: 19.8s
95'th percentile: 44.3s
99'th percentile: 81.5s
99.9'th percentile: 143.6s
99.99'th percentile: 181.0s
99.999'th percentile: 185.8s
samples=15086, minimum=10.1s, maximum=185.8s, mean=21.4s, stdev=13.4s

Hotmail
50'th percentile: 20.2s
68'th percentile: 24.4s
95'th percentile: 79.3s
99'th percentile: 139.7s
99.9'th percentile: 180.8s
99.99'th percentile: 184.9s
99.999'th percentile: 188.6s
samples=14989, minimum=11.1s, maximum=188.6s, mean=29.3s, stdev=24.3s

Yahoo!
50'th percentile: 16.1s
68'th percentile: 17.5s
95'th percentile: 43.0s
99'th percentile: 88.4s
99.9'th percentile: 156.9s
99.99'th percentile: 180.9s
99.999'th percentile: 180.9s
samples=14992, minimum=9.0s, maximum=180.9s, mean=19.7s, stdev=14.1s
```

[HdrHistogram]: https://github.com/HdrHistogram/HdrHistogram
[hdrhistogram crate]: https://docs.rs/hdrhistogram/latest/hdrhistogram/
