+++
title = "Installing Kopia on Nixos"
description = "Making backups with Kopia is easy. Plus, it comes with all kinds of useful features that save you money"
date = 2023-10-30
updated = 2023-11-12
draft = false

[taxonomies]
tags = ["Data","Linux","NixOS","Tutorial"]
[extra]
toc = true
+++
I have a bunch of data that I want to backup -- who doesn't? Here are my considerations and why I landed on *Kopia*:

- Data lives on many hosts and I want all of it to go to the same place. The backup solution should be able to take advantage of this by getting a better deduplication ratio.
- Everything needs to be encrypted to protect against data breaches and so that I can upload to any host, even untrusted ones.
- Version history should be maintained and easily explorable.
- Compression must be supported, as I keep many easily compressible (text) files.
- Must be open source.

*Kopia* satisfies all of these requirements, plus it provides many niceties that one would expect from a cloud native application such as caching backups / metadata to save costs. It also provides a server mode so that each host can backup their files without potentially interacting with the other hosts' files. This is nice from a security perspective, since in theory a host can be compromised without potentially compromising all my backups. That's just theoretically though, and I'm pretty sure that *Kopia* was not written with security at the forefront. It does ensure that I don't accidentally mess with backups that I didn't want to touch.

Anyway, enough about *Kopia*. Let's set it up.

## Setting up *Kopia*

For the initial setup, I install *Kopia* on a dedicated machine. This machine will receive and handle all of the requests from the different hosts, so it should have at least a few resources. Setting up the other hosts will come later.

### Installation

I'll be setting up *Kopia* through its CLI on *NixOS* version 23.05. Some day I will write declarative configuration for it, but that day is not today.

To install Kopia, add `kopia` to your system packages. In my case, I'm using *Home Manager*, so my config looks something like:

```nix
home.packages = with pkgs; [
  # ... other packages
  kopia
];
```

Then I just run `sudo nixos-rebuild switch` to rebuild my *Home Manager* config and install the software.


### Repository creation

I'll be using a *Backblaze B2* bucket configured as an *S3* bucket. This, along with the other repository types, can be seen [here](https://kopia.io/docs/repositories/#kopia-cli). The reason for configuring as an *S3* bucket is that some features such as object lock do not work with with the *B2* bucket interface. Plus, it saves re-configuring as I can use *S3* for many different cloud providers.

I create the repository [with](https://kopia.io/docs/reference/command-line/common/repository-create-s3/):

```sh
kopia repository create s3 \
        --bucket=gtbbackupsall \
        --endpoint=s3.us-east-005.backblazeb2.com \
        --access-key=005b8442ecf5a4f0000000002 \
        --secret-access-key=#secret!
```

The command will ask for a bunch of stuff -- I just leave them all at their defaults. It will ask for a repository password which should be secure and is used for the encryption. It can be changed later.

To connect to the repository later, just use connect rather than create.

### Server mode

This dedicated host will be running in server mode so that the other hosts can access the repository using some arbitrary login credentials rather than the encryption key. This mode is flexible and has customizable access control list rules, should that be something you need. I find that the defaults work fine for my use.

The main reference for this section is [here](https://kopia.io/docs/repository-server/).

I'll be adding a user named `immich` on the host `immich`.

```sh
kopia server user add immich@immich
```

This will prompt for a password, which allows for users to access the repository without having the repository's private key.

To start the server with an auto-generated TLS certificate for the first time, I run:

```sh
kopia server start \
  --tls-generate-cert \
  --tls-cert-file ~/kopia.cert \
  --tls-key-file ~/kopia.key \
  --address [HOST_IP]:51515 \
  --server-control-username control \
  --server-control-password [PASSWORD_HERE]
```

Of course, you can use your own certificates if you want. 

This will return the fingerprint of the certificate. If you missed it, just put a dummy value in and Kopia will tell you the real hash when you [try and fail to connect](https://kopia.discourse.group/t/how-to-get-kopia-server-sha/1490). Otherwise you can use `openssl` to get the fingerprint, but it's honestly easier to just do it by failing to connect.

For future launches of the server, ensure that you run the command without `--tls-generate-cert`.

### Access control list

Now that I have a user, I want to set up the ACL to determine what they are allowed to do. The [default](https://kopia.io/docs/repository-server/#server-access-control-acl) is for the user to only be able to:

- read and write policies for `username@hostname` and `username@hostname:/path`,
- read and write snapshots for `username@hostname:/path`,
- read `global` policy,
- read `host`-level policies for their own `hostname`,
- read and write `user`-level policies for their own `username@hostname`,
- read and write their own `user` account `username@hostname` (to be able to change password),
- read objects if they know their object IDs
- write new `content` to the repository

This is actually pretty tight, and it's not obvious whether any benefit would come from tightening it further. At the very least it will keep me from accidentally modifying something that I didn't mean to.

### Connecting

To connect to the server from the `immich` host I run:

```sh
kopia repository connect server \
  --url https://[SERVER_ADDRESS]:51515 \
  --server-cert-fingerprint [FINGERPRINT] \
  --override-username=immich \
  --override-hostname=immich
```

### Defining policy

Now that I'm connected to the repository on a computer with files that I want backed up, I need to tell Kopia how to back those files up.

The [docs](https://kopia.io/docs/getting-started/#policies) sum up `policy` better than I can:

> I can change policy settings using the [`kopia policy set` command](https://kopia.io/docs/reference/command-line/common/policy-set/). This command allows you to change the `global` policy or change specific policies for a ‘user@host’, a ‘@host’, a ‘user@host:path’, or a particular directory. For example, here I tell Kopia to set the policy to ignore two directories from being included in the snapshot of `jarek@jareks-mbp:/Users/jarek/Projects/Kopia/site`:

There's just one policy that I will be modifying before creating my first snapshot. I want to [exclude files and directories](https://kopia.discourse.group/t/how-to-ignore-directories/497) from being snapshotted. To do that I run:

```sh
kopia policy set [directory to be snapshotted] \
  --add-ignore [file glob] \
  --add-ignore [another file glob]
```

Alternatively, you can add a `.kopiaignore` file to the directory and add rules to it the same as a `.gitignore`.

To check that only the thing you want will be snapshotted, run:

```sh
kopia snapshot estimate [path]
```

The only other policy that I might want to touch is how many snapshots to keep, but I can take care of that later.

### Compression (optional)

Compression has [good support](https://kopia.io/docs/advanced/compression/) in *Kopia*, though there is still room for improvement as the algorithm for whether to compress a file or not is very rudimentary. 

To get started, I'd recommend benchmarking the algorithms on files that you're going to be including in snapshots. If you already know what algorithm you want to use, then take a look at [this table](https://kopia.io/docs/advanced/compression/#algorithm) for the name that *Kopia* uses for the algorithm. Then look to the end of this section for the command to add compression to the snapshot policy.

Get started by adding a data directory to a .tar archive with:

```sh
tar -cvf test.tar [directory]
```

Note this doesn't exclude files that are ignored by Kopia. So if your ignore list is significant, this may give skewed results.

Now benchmark every algorithm [with](https://kopia.io/docs/reference/command-line/common/benchmark-compression/):

```sh
kopia benchmark compression \
  --data-file=test.tar \
  < --by-size or --by-alloc >
```

The output will look something like:

```txt
    Compression                Compressed   Throughput   Allocs   Usage
--------------------------------------------------------------------------
 0. zstd-better-compression    17.8 MB      210 MB/s     3163     99.7 MB
 1. gzip-best-compression      18 MB        34.9 MB/s    44       67.9 MB
 2. pgzip-best-compression     18 MB        183.8 MB/s   1549     143.6 MB
 3. deflate-best-compression   18 MB        40.1 MB/s    40       68.3 MB
 4. zstd                       18 MB        455.3 MB/s   3135     88.7 MB
 5. gzip                       18 MB        54.7 MB/s    41       67.9 MB
 6. zstd-fastest               18.3 MB      618.4 MB/s   6155     52.6 MB
 7. deflate-default            18.3 MB      281.5 MB/s   39       68.2 MB
 8. pgzip                      18.3 MB      1.1 GB/s     1479     130.1 MB
 9. s2-better                  18.6 MB      2.5 GB/s     1079     81.3 MB
10. pgzip-best-speed           18.7 MB      1.9 GB/s     1398     100.6 MB
11. deflate-best-speed         18.7 MB      459.9 MB/s   37       67.9 MB
12. gzip-best-speed            18.8 MB      264.2 MB/s   46       68.3 MB
13. s2-parallel-4              19.1 MB      2.8 GB/s     988      66.9 MB
14. s2-parallel-8              19.1 MB      3.6 GB/s     1076     80.7 MB
15. s2-default                 19.1 MB      4 GB/s       1130     84.9 MB
```

`Compressed` is the final compressed size of the file.

`Throughput` is how much data can be compressed per second. Note that this assumes unlimited disk I/O speed. If you're snapshotting data from a spinning rust hard-drive, you probably can't hit the 4 GB/s that s2-default can theoretically provide. 

`Allocs` is the number of memory allocations needed during compression. If you know why this is important, please drop me a line!

`Usage` is the total memory usage during compression. If your host is extremely memory constrained, this might be an important factor.

Out of those, the only thing I look at is `compressed`. So I'll end up choosing `zstd-better-compression` almost every time. There used to be a `zstd-best-compression`, but that was deprecated for some reason, despite providing better compression ratio. The reason for this I'm not sure, but it's probably best to not use deprecated features.

Finally, add compression using your algorithm of choice to the policy with:

```sh
kopia policy set [path] --compression=[algorithm]
```

### Snapshotting directories

Now with everything ready, I [create a snapshot](https://kopia.io/docs/getting-started/#incremental-snapshots) with:

```sh
kopia snapshot create [path]
```

Assuming this succeeds, all that's left to do is configure automatic snapshots. You can use `cron` or `systemd services` to accomplish this -- *Kopia* apparently has something built in for scheduling backups, but one of the creators themselves [doesn't use it](https://kopia.discourse.group/t/policy-schedule/74), so I'm not going to bother with it.

Since NixOS already uses *systemd* extensively, I'm going to use a *systemd service* which gets triggered periodically by a *systemd timer* to automatically trigger snapshots. There are two main ways to do this -- either through *Home Manager* or through the global config. For *Home Manager*, I add something like [this](https://discourse.nixos.org/t/is-there-a-trick-to-getting-systemd-user-timers-to-work/27997) to my *Home Manager* config:

```nix
systemd.user.services.kopia = {
  Unit = {
    Description = "Kopia backup";
    After = [ "network.target" ];
  };
  Service = {
    Type = "oneshot";
    ExecStart = toString (
      pkgs.writeShellScript "kopia-backup-script.sh" ''
        set -eou pipefail

        ${pkgs.kopia}/bin/kopia snapshot create [SNAPSHOT_DIR1]
        ${pkgs.kopia}/bin/kopia snapshot create [SNAPSHOT_DIR2]
      ''
    );
  };
};

systemd.user.timers.kopia = {
  Unit.Description = "Kopia backup schedule";
  Timer = {
    Unit = "oneshot";
    OnBootSec = "1h";
    OnUnitActiveSec = "1h";
  };
  Install.WantedBy = [ "timers.target" ];
};
```

For global config, I use something like [this](https://nixos.wiki/wiki/Systemd/Timers):

```nix
  systemd.services.kopia = {
    after = [ "network.target" ];
    serviceConfig = {
      Type = "oneshot";
      User = "root";
      ExecStart = toString (
        pkgs.writeShellScript "kopia-backup-script.sh" ''
          set -eou pipefail

          ${pkgs.kopia}/bin/kopia snapshot create [SNAPSHOT_DIR1]
          ${pkgs.kopia}/bin/kopia snapshot create [SNAPSHOT_DIR2]
        ''
      );
    };
  };

  systemd.timers.kopia = {
    timerConfig = {
      OnBootSec = "1h";
      OnUnitActiveSec = "1h";
      Unit = "kopia.service";
    };
    wantedBy = [ "timers.target" ];
  };
```

That's pretty much it! If all went well then snapshots should be quick and painless to add, and you can rest easy. Keep an eye out for a followup article on the second mandatory step for a good backup policy -- periodically verifying that backups are being taken, are accessible, and can be restored from quickly.

## Further reading

[https://kopia.discourse.group/t/configure-multiple-repositories/1194/2/](https://kopia.discourse.group/t/configure-multiple-repositories/1194/2/)

[https://kopia.io/docs/faqs/](https://kopia.io/docs/faqs/)
