# Fetching Resources

## Build

We need to run the build in a privileged container to be able to mount the
file system to edit the files.

```sh
docker build -t risc-v-sbi-linux .
docker run --rm --privileged -v "$(pwd):/artifacts" risc-v-sbi-linux:latest
```
