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
99'th percentile: 30.7s
99.9'th percentile: 84.2s
99.99'th percentile: 174.8s
99.999'th percentile: 174.8s
samples=6075, minimum=9.6s, maximum=174.8s, mean=15.9s, stdev=5.3s

Apple
50'th percentile: 66.3s
68'th percentile: 83.4s
95'th percentile: 175.1s
99'th percentile: 180.7s
99.9'th percentile: 181.0s
99.99'th percentile: 223.7s
99.999'th percentile: 223.7s
samples=6071, minimum=13.1s, maximum=223.7s, mean=69.2s, stdev=47.4s

Gmail
50'th percentile: 17.8s
68'th percentile: 20.6s
95'th percentile: 50.4s
99'th percentile: 88.0s
99.9'th percentile: 158.2s
99.99'th percentile: 185.8s
99.999'th percentile: 185.8s
samples=6071, minimum=10.1s, maximum=185.8s, mean=22.5s, stdev=14.9s

Hotmail
50'th percentile: 20.4s
68'th percentile: 27.1s
95'th percentile: 95.5s
99'th percentile: 152.4s
99.9'th percentile: 180.9s
99.99'th percentile: 188.6s
99.999'th percentile: 188.6s
samples=6070, minimum=11.1s, maximum=188.6s, mean=32.4s, stdev=28.3s

Yahoo!
50'th percentile: 16.1s
68'th percentile: 17.5s
95'th percentile: 45.9s
99'th percentile: 96.5s
99.9'th percentile: 173.9s
99.99'th percentile: 180.8s
99.999'th percentile: 180.8s
samples=6072, minimum=9.0s, maximum=180.8s, mean=20.2s, stdev=15.2s
```

[HdrHistogram]: https://github.com/HdrHistogram/HdrHistogram
[hdrhistogram crate]: https://docs.rs/hdrhistogram/latest/hdrhistogram/
