# ANEW - Append New Lines to files

Append only new lines from stdin. If lines from input are already present in the original file, 
nothing is done. 

This tool works exactly as using `tee -a`: new lines are appended to the file and written to stdout.

## Install

```shell
cargo install aanew
```

## Usage

```sh
$ cat my-old-file-without-the-new-line
old line 1
old line 2
old line 3
old line 4
old line 5
old line 6

$ echo "a new line to append" | anew my-old-file-without-the-new-line
# next line was appended to `my-old-file-without-the-new-line` file
a new line to append 

$ cat my-old-file-without-the-new-line
old line 1
old line 2
old line 3
old line 4
old line 5
old line 6
a new line to append
```
