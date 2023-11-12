# Fetching Resources

## Build

```sh
docker build -t risc-v-sbi-linux .
docker run --rm --privileged -v "$(pwd):/artifacts" risc-v-sbi-linux:latest
```
