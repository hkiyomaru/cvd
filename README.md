# cvd

**cvd** lists available GPU device IDs. The output can be used to set `CUDA_VISIBLE_DEVICES` like this:

```sh
$ env CUDA_VISIBLE_DEVICES=$(cvd) python
```

## Command-line options

- **-n=(num)**: Limits the number of GPUs to show.
- **-e, --empty-only**: Excludes GPUs on which a job is running.
