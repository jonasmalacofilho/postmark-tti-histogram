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
50'th percentile: 15.5s
68'th percentile: 16.7s
95'th percentile: 23.8s
99'th percentile: 61.6s
99.9'th percentile: 100.5s
99.99'th percentile: 180.6s
99.999'th percentile: 181.0s
samples=22972, minimum=8.4s, maximum=181.0s, mean=17.3s, stdev=8.7s

Apple
50'th percentile: 30.2s
68'th percentile: 72.4s
95'th percentile: 147.4s
99'th percentile: 180.6s
99.9'th percentile: 181.0s
99.99'th percentile: 194.9s
99.999'th percentile: 223.7s
samples=22837, minimum=11.4s, maximum=223.7s, mean=55.5s, stdev=44.5s

Gmail
50'th percentile: 17.2s
68'th percentile: 19.2s
95'th percentile: 47.0s
99'th percentile: 76.9s
99.9'th percentile: 133.7s
99.99'th percentile: 180.5s
99.999'th percentile: 185.8s
samples=22959, minimum=8.7s, maximum=185.8s, mean=21.1s, stdev=13.1s

Hotmail
50'th percentile: 19.9s
68'th percentile: 23.0s
95'th percentile: 71.7s
99'th percentile: 127.0s
99.9'th percentile: 180.7s
99.99'th percentile: 184.9s
99.999'th percentile: 188.6s
samples=22854, minimum=10.4s, maximum=188.6s, mean=27.6s, stdev=21.9s

Yahoo!
50'th percentile: 16.3s
68'th percentile: 17.7s
95'th percentile: 46.4s
99'th percentile: 82.6s
99.9'th percentile: 161.7s
99.99'th percentile: 180.9s
99.999'th percentile: 190.1s
samples=22871, minimum=6.4s, maximum=190.1s, mean=20.0s, stdev=14.0s
```

[HdrHistogram]: https://github.com/HdrHistogram/HdrHistogram
[hdrhistogram crate]: https://docs.rs/hdrhistogram/latest/hdrhistogram/
