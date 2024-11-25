# Collect and analyze Postmark Time to Inbox

[Postmark] measures the time it takes for an email to enter their pipeline to it being (reported as)
delivered to the user's inbox. This is reportedly done at 5-min intervals against a few of the major
email services.

A summary of the data from the last 24 hours is published on [tti.postmarkapp.com], and the data
shown there is fetched from the [tti.postmarkapp.com/api/1.0/tti] endpoint.

But I would like to look at the data over a longer time frame than just the last 24 hours. So let's
collect the data from that endpoint and try to analyze it.

_Note that the data we get appears to consist of averages of an unknown number of the true
measurements. At 5-min intervals, we would expected 288 data points per day, but we only get 96._

[Postmark]: https://postmarkapp.com/
[tti.postmarkapp.com]: tti.postmarkapp.com
[tti.postmarkapp.com/api/1.0/tti]: tti.postmarkapp.com/api/1.0/tti

## Collection

Done using a simple systemd timer, see [`./systemd`](./systemd). Don't forget to adjust variables and
paths.

## Analysis

The idea is to throw the data into [HdrHistogram] (specifically, the [hdrhistogram] crate). Latency
data like this is known to almost always be non-normal, and we also care more about the bad cases
than the good ones. HdrHistogram is one of standard and easier ways to look at this type of data.

One issue is that the data we collect contains redundant values. My idea is to use a global
alignment algorithm (e.g. Needleman-Wunsch) to find the duplicated data and throw it away.

Some of this is implemented, the rest is still to-do...

## Example

```
$ cargo run <path to data file>...
AOL:
50'th percentile: 19.1s
68'th percentile: 21.0s
95'th percentile: 64.0s
99'th percentile: 104.5s
99.9'th percentile: 104.5s
99.99'th percentile: 104.5s
99.999'th percentile: 104.5s
samples=96, minimum=13.9s, maximum=104.5s, mean=24.6s, stdev=17.0s

Apple:
50'th percentile: 24.2s
68'th percentile: 57.9s
95'th percentile: 89.2s
99'th percentile: 180.6s
99.9'th percentile: 180.6s
99.99'th percentile: 180.6s
99.999'th percentile: 180.6s
samples=96, minimum=13.7s, maximum=180.6s, mean=41.1s, stdev=29.8s

Gmail:
50'th percentile: 15.5s
68'th percentile: 17.0s
95'th percentile: 20.0s
99'th percentile: 23.2s
99.9'th percentile: 23.2s
99.99'th percentile: 23.2s
99.999'th percentile: 23.2s
samples=96, minimum=12.5s, maximum=23.2s, mean=16.0s, stdev=2.3s

Hotmail:
50'th percentile: 16.4s
68'th percentile: 18.0s
95'th percentile: 45.9s
99'th percentile: 109.4s
99.9'th percentile: 109.4s
99.99'th percentile: 109.4s
99.999'th percentile: 109.4s
samples=96, minimum=10.8s, maximum=109.4s, mean=19.5s, stdev=13.7s

Yahoo!:
50'th percentile: 16.0s
68'th percentile: 17.3s
95'th percentile: 54.0s
99'th percentile: 73.0s
99.9'th percentile: 73.0s
99.99'th percentile: 73.0s
99.999'th percentile: 73.0s
samples=96, minimum=12.5s, maximum=73.0s, mean=19.0s, stdev=11.2s
```

[HdrHistogram]: https://github.com/HdrHistogram/HdrHistogram
[hdrhistogram crate]: https://docs.rs/hdrhistogram/latest/hdrhistogram/
