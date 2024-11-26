# Collect and analyze Postmark Time to Inbox

[Postmark] measures the time it takes for an email to enter their pipeline to it being (reported as)
delivered to the user's inbox. This is reportedly done at 5-min intervals against a few of the major
email services.

A summary of the data from the last 24 hours is published on [tti.postmarkapp.com], and the data
shown there is fetched from the [tti.postmarkapp.com/api/1.0/tti] endpoint. This data has 15-minute
granularity.

I would like to look at this data over a longer time frame than just the last 24 hours. So let's
collect the data from that endpoint and try to analyze it.

[Postmark]: https://postmarkapp.com/
[tti.postmarkapp.com]: tti.postmarkapp.com
[tti.postmarkapp.com/api/1.0/tti]: tti.postmarkapp.com/api/1.0/tti

## Collection

Done using a simple systemd timer, see [`./systemd`](./systemd). Don't forget to adjust variables
and paths.

## Analysis

For now, the data is only thrown into an [HdrHistogram] (specifically, using the [hdrhistogram]
crate), and a few basic metrics are printed.

Latency data like this is known to almost always be non-normal, and we also care more about the bad
cases than the good ones. HdrHistogram is one of standard and easier ways to look at this type of
data.

## Example

```
$ cargo run <directory>
AOL
50'th percentile: 15.7s
68'th percentile: 16.9s
95'th percentile: 22.2s
99'th percentile: 37.8s
99.9'th percentile: 63.3s
99.99'th percentile: 63.3s
99.999'th percentile: 63.3s
samples=266, minimum=8.4s, maximum=63.3s, mean=16.6s, stdev=4.8s

Apple
50'th percentile: 25.6s
68'th percentile: 59.8s
95'th percentile: 125.8s
99'th percentile: 180.6s
99.9'th percentile: 185.9s
99.99'th percentile: 185.9s
99.999'th percentile: 185.9s
samples=266, minimum=13.6s, maximum=185.9s, mean=46.4s, stdev=36.3s

Gmail
50'th percentile: 17.3s
68'th percentile: 19.9s
95'th percentile: 54.0s
99'th percentile: 101.4s
99.9'th percentile: 106.7s
99.99'th percentile: 106.7s
99.999'th percentile: 106.7s
samples=266, minimum=10.4s, maximum=106.7s, mean=21.7s, stdev=14.2s

Hotmail
50'th percentile: 19.4s
68'th percentile: 24.6s
95'th percentile: 94.7s
99'th percentile: 120.9s
99.9'th percentile: 180.0s
99.99'th percentile: 180.0s
99.999'th percentile: 180.0s
samples=265, minimum=10.6s, maximum=180.0s, mean=31.2s, stdev=25.8s

Yahoo!
50'th percentile: 16.4s
68'th percentile: 18.1s
95'th percentile: 51.2s
99'th percentile: 88.9s
99.9'th percentile: 109.4s
99.99'th percentile: 109.4s
99.999'th percentile: 109.4s
samples=266, minimum=9.6s, maximum=109.4s, mean=21.2s, stdev=14.6s
```

[HdrHistogram]: https://github.com/HdrHistogram/HdrHistogram
[hdrhistogram crate]: https://docs.rs/hdrhistogram/latest/hdrhistogram/
