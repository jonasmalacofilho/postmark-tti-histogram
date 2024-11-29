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
68'th percentile: 16.8s
95'th percentile: 23.2s
99'th percentile: 51.7s
99.9'th percentile: 174.8s
99.99'th percentile: 174.8s
99.999'th percentile: 174.8s
samples=611, minimum=8.4s, maximum=174.8s, mean=17.2s, stdev=9.5s

Apple
50'th percentile: 28.9s
68'th percentile: 72.4s
95'th percentile: 147.0s
99'th percentile: 180.6s
99.9'th percentile: 189.8s
99.99'th percentile: 189.8s
99.999'th percentile: 189.8s
samples=612, minimum=13.6s, maximum=189.8s, mean=55.1s, stdev=44.3s

Gmail
50'th percentile: 16.9s
68'th percentile: 19.6s
95'th percentile: 62.0s
99'th percentile: 98.4s
99.9'th percentile: 180.6s
99.99'th percentile: 180.6s
99.999'th percentile: 180.6s
samples=611, minimum=10.4s, maximum=180.6s, mean=23.0s, stdev=18.2s

Hotmail
50'th percentile: 20.6s
68'th percentile: 30.2s
95'th percentile: 112.4s
99'th percentile: 162.3s
99.9'th percentile: 180.9s
99.99'th percentile: 180.9s
99.999'th percentile: 180.9s
samples=610, minimum=10.6s, maximum=180.9s, mean=37.2s, stdev=33.7s

Yahoo!
50'th percentile: 16.8s
68'th percentile: 19.0s
95'th percentile: 83.7s
99'th percentile: 127.0s
99.9'th percentile: 181.0s
99.99'th percentile: 181.0s
99.999'th percentile: 181.0s
samples=610, minimum=8.3s, maximum=181.0s, mean=26.5s, stdev=24.6s
```

[HdrHistogram]: https://github.com/HdrHistogram/HdrHistogram
[hdrhistogram crate]: https://docs.rs/hdrhistogram/latest/hdrhistogram/
