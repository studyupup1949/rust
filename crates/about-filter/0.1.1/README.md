# about-filter

[cgit](https://git.zx2c4.com/cgit/) expects a `README` formatter with a peculiar
mode of operation, where it receives a “filename” as its first argument, but
doesn't open it. Instead, it's meant to use it as a cue to determine the format,
then read from standard input, on which cgit will provide the `README` it
extracts from the repository by whatever magic `libgit` call. The included
formatter is a shell script that makes further calls out to assorted python
scripts. This replaces that whole mess with a single statically-linkable
executable, so in your `cgitrc` you can write:

    about-filter=/whatever/bin/about-filter
    
and keep your `chroot` or container or what-have-you nice and small.

Functionality is equivalent to the script(s) included with cgit, with the
exception of manpage support.
