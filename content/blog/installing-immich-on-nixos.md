+++
title = "Installing Immich on Nixos"
description = """
  Immich is a self-hosted alternative to Google Photos
  that's growing incredibly quickly.
"""
date = 2024-06-02
draft = true

[taxonomies]
tags = ["Data","Linux","NixOS","Tutorial"]
[extra]
toc = true
+++

*Immich* is a self-hosted alternative to *Google Photos* (the app that shall not be named). I've used it for over a year now and I'm amazed at the rate that the project grows.

Standard deployment of *Immich* is done through *Docker Compose* and the app is split into different microservices. This makes it a challenge to port it to *NixOS*, and there isn't yet a package for it. I used to run it in a VM with the recommended setup, but recently some very unfortunate events led to parts of the VM disk becoming corrupted, which forced me to setup *Immich* once again.

This time I'm setting up Immich directly on the host computer to avoid a similar situation from ocurring again. It turns out that managing containers declaratively with *nix* is pretty easy, albeit with some gotchas.

# Containers in NixOS

In full, my configuration is

```nix
{
  config,
  pkgs,
  ...
}: let
  immichHost = "immich.services.gabrieltb.me";

  immichRoot = "/immich";
  immichPhotos = "${immichRoot}/photos";
  immichAppdataRoot = "${immichRoot}/appdata";
  immichVersion = "v1.105.1";

  postgresRoot = "${immichAppdataRoot}/pgsql";
  postgresPassword = "hunter2"; # TODO: use secrets (is this necessary?)
  postgresUser = "immich";
  postgresDb = "immich";

  extraOptions = [
    "--pull=newer"
    "--network=immich-network"
  ];
in {
  virtualisation = {
    containers.enable = true;
    podman = {
      enable = true;
      defaultNetwork.settings.dns_enabled = true;
    };
  };

  # To fix podman DNS traffic.
  networking.firewall.interfaces."podman+".allowedUDPPorts = [53 5353];
  # To make redis happy.
  boot.kernel.sysctl = {"vm.overcommit_memory" = 1;};

  systemd.services.create-immich-network = with config.virtualisation.oci-containers; {
    serviceConfig.Type = "oneshot";
    wantedBy = [
      "${backend}-immich_server.service"
      "${backend}-immich_microservices.service"
      "${backend}-immich_machine_learning.service"
      "${backend}-immich_redis.service"
      "${backend}-immich_postgres.service"
    ];
    script = ''
      ${pkgs.podman}/bin/podman network exists immich-network || \
      ${pkgs.podman}/bin/podman network create immich-network
    '';
  };

  services.nginx.enable = true;
  services.nginx.virtualHosts."${immichHost}" = {
    extraConfig = ''
      ## Per https://immich.app/docs/administration/reverse-proxy...
      client_max_body_size 50000M;
    '';
    forceSSL = true;
    enableACME = true;
    locations."/" = {
      proxyPass = "http://127.0.0.1:2283";
      proxyWebsockets = true;
    };
  };
  security.acme = {
    acceptTerms = true;
    defaults.email = "gabriel@gabrieltb.me";
  };
  networking.firewall.allowedTCPPorts = [80 443];

  # The primary source for this configuration is the recommended docker-compose installation of immich from
  # https://immich.app/docs/install/docker-compose, which linkes to:
  # - https://github.com/immich-app/immich/releases/latest/download/docker-compose.yml
  # - https://github.com/immich-app/immich/releases/latest/download/example.env
  # and has been transposed into nixos configuration here.  Those upstream files should be checked
  # for serious changes if there are any upgrade problems here.
  #
  # After initial deployment, these in-process configurations need to be done:
  # - create an admin user by accessing the site
  # - login with the admin user
  # - set the "Machine Learning Settings" > "URL" to http://immich_machine_learning:3003

  virtualisation.oci-containers.containers = {
    immich_server = {
      image = "ghcr.io/immich-app/immich-server:${immichVersion}";
      cmd = ["start.sh" "immich"];
      volumes = [
        "${immichPhotos}:/usr/src/app/upload"
        "/etc/localtime:/etc/localtime:ro"
      ];
      environment = {
        IMMICH_VERSION = immichVersion;
        DB_HOSTNAME = "immich_postgres";
        DB_USERNAME = postgresUser;
        DB_DATABASE_NAME = postgresDb;
        DB_PASSWORD = postgresPassword;
        REDIS_HOSTNAME = "immich_redis";
      };
      ports = ["127.0.0.1:2283:3001"];
      dependsOn = [
        "immich_redis"
        "immich_postgres"
      ];
      extraOptions = extraOptions;
    };

    immich_microservices = {
      image = "ghcr.io/immich-app/immich-server:${immichVersion}";
      cmd = ["start.sh" "microservices"];
      volumes = [
        "${immichPhotos}:/usr/src/app/upload"
        "/etc/localtime:/etc/localtime:ro"
      ];
      environment = {
        IMMICH_VERSION = immichVersion;
        DB_HOSTNAME = "immich_postgres";
        DB_USERNAME = postgresUser;
        DB_DATABASE_NAME = postgresDb;
        DB_PASSWORD = postgresPassword;
        REDIS_HOSTNAME = "immich_redis";
      };
      dependsOn = [
        "immich_redis"
        "immich_postgres"
      ];
      extraOptions = extraOptions;
    };

    immich_machine_learning = {
      image = "ghcr.io/immich-app/immich-machine-learning:${immichVersion}";
      volumes = ["${immichAppdataRoot}/model-cache:/cache"];
      environment = {
        IMMICH_VERSION = immichVersion;
      };
      extraOptions = extraOptions;
    };

    immich_redis = {
      image = "registry.hub.docker.com/library/redis:6.2-alpine@sha256:84882e87b54734154586e5f8abd4dce69fe7311315e2fc6d67c29614c8de2672";
      extraOptions = extraOptions;
    };

    immich_postgres = {
      image = "registry.hub.docker.com/tensorchord/pgvecto-rs:pg14-v0.2.0@sha256:90724186f0a3517cf6914295b5ab410db9ce23190a2d9d0b9dd6463e3fa298f0";
      volumes = ["${postgresRoot}:/var/lib/postgresql/data"];
      environment = {
        POSTGRES_PASSWORD = postgresPassword;
        POSTGRES_USER = postgresUser;
        POSTGRES_DB = postgresDb;
        POSTGRES_INITDB_ARGS = "--data-checksums";
      };
      extraOptions = extraOptions;
    };
  };
}
```

Please note that most my my configuration has come from (this gist)[https://gist.github.com/mfenniak/c6f6b1cde07fc33df4d925e13f7d5afa] and (this blog post)[https://kressle.in/immich].






Using the `virtualisation.oci-containers.containers` configuration, we can declare containers that will be managed by systemd. For me, this looked like:

```nix
virtualisation.oci-containers.containers = {
  immich_server = {
    image = "ghcr.io/immich-app/immich-server:${immichVersion}";
    cmd = ["start.sh" "immich"];
    volumes = [
      "${immichPhotos}:/usr/src/app/upload"
      "/etc/localtime:/etc/localtime:ro"
    ];
    environment = {
      IMMICH_VERSION = immichVersion;
      DB_HOSTNAME = "immich_postgres";
      DB_USERNAME = postgresUser;
      DB_DATABASE_NAME = postgresDb;
      DB_PASSWORD = postgresPassword;
      REDIS_HOSTNAME = "immich_redis";
    };
    ports = ["127.0.0.1:2283:3001"];
    dependsOn = [
      "immich_redis"
      "immich_postgres"
    ];
    extraOptions = extraOptions;
  };

  immich_microservices = {
    image = "ghcr.io/immich-app/immich-server:${immichVersion}";
    cmd = ["start.sh" "microservices"];
    volumes = [
      "${immichPhotos}:/usr/src/app/upload"
      "/etc/localtime:/etc/localtime:ro"
    ];
    environment = {
      IMMICH_VERSION = immichVersion;
      DB_HOSTNAME = "immich_postgres";
      DB_USERNAME = postgresUser;
      DB_DATABASE_NAME = postgresDb;
      DB_PASSWORD = postgresPassword;
      REDIS_HOSTNAME = "immich_redis";
    };
    dependsOn = [
      "immich_redis"
      "immich_postgres"
    ];
    extraOptions = extraOptions;
  };

  immich_machine_learning = {
    image = "ghcr.io/immich-app/immich-machine-learning:${immichVersion}";
    volumes = ["${immichAppdataRoot}/model-cache:/cache"];
    environment = {
      IMMICH_VERSION = immichVersion;
    };
    extraOptions = extraOptions;
  };

  immich_redis = {
    image = "registry.hub.docker.com/library/redis:6.2-alpine@sha256:84882e87b54734154586e5f8abd4dce69fe7311315e2fc6d67c29614c8de2672";
    extraOptions = extraOptions;
  };

  immich_postgres = {
    image = "registry.hub.docker.com/tensorchord/pgvecto-rs:pg14-v0.2.0@sha256:90724186f0a3517cf6914295b5ab410db9ce23190a2d9d0b9dd6463e3fa298f0";
    volumes = ["${postgresRoot}:/var/lib/postgresql/data"];
    environment = {
      POSTGRES_PASSWORD = postgresPassword;
      POSTGRES_USER = postgresUser;
      POSTGRES_DB = postgresDb;
      POSTGRES_INITDB_ARGS = "--data-checksums";
    };
    extraOptions = extraOptions;
  };
};
```

At the top of my configuration lives the variables that I use multiple times throughout my config:

```nix
{
  config,
  pkgs,
  ...
}: let
  immichHost = "immich.services.gabrieltb.me";

  immichRoot = "/immich";
  immichPhotos = "${immichRoot}/photos";
  immichAppdataRoot = "${immichRoot}/appdata";
  immichVersion = "v1.105.1";

  postgresRoot = "${immichAppdataRoot}/pgsql";
  postgresPassword = "hunter2";
  postgresUser = "immich";
  postgresDb = "immich";

  extraOptions = [
    "--pull=newer"
    "--network=immich-network"
  ];
in {
  ...
}
```

then I enable virtualisation:

```nix
virtualisation = {
  containers.enable = true;
  podman = {
    enable = true;
    defaultNetwork.settings.dns_enabled = true;
  };
};
```

the crux of the issue I faced was that DNS traffic was not allowed over the podman network interface.

```nix
networking.firewall.interfaces."podman+".allowedUDPPorts = [53 5353];
```

To stop redis from complaining (and apparently to prevent horrible corruption if I run out of ram), I add

```nix
boot.kernel.sysctl = {"vm.overcommit_memory" = 1;};
```

To automatically create the immich network I make a systemd service:

```nix
systemd.services.create-immich-network = with config.virtualisation.oci-containers; {
  serviceConfig.Type = "oneshot";
  wantedBy = [
    "${backend}-immich_server.service"
    "${backend}-immich_microservices.service"
    "${backend}-immich_machine_learning.service"
    "${backend}-immich_redis.service"
    "${backend}-immich_postgres.service"
  ];
  script = ''
    ${pkgs.podman}/bin/podman network exists immich-network || \
    ${pkgs.podman}/bin/podman network create immich-network
  '';
};
```

I choose to reverse proxy *Immich* directly, and use *NGINX* for that:

```nix
services.nginx.enable = true;
services.nginx.virtualHosts."${immichHost}" = {
  extraConfig = ''
    ## Per https://immich.app/docs/administration/reverse-proxy...
    client_max_body_size 50000M;
  '';
  forceSSL = true;
  enableACME = true;
  locations."/" = {
    proxyPass = "http://127.0.0.1:2283";
    proxyWebsockets = true;
  };
};
security.acme = {
  acceptTerms = true;
  defaults.email = "gabriel@gabrieltb.me";
};
networking.firewall.allowedTCPPorts = [80 443];
```





Issues with dns highlighted.
https://www.reddit.com/r/NixOS/comments/13e5w6b/does_anyone_have_a_working_nixos_ocicontainers/

Nixpkg blocked because it's hard to make immich work on nixos and the developers don't use nix.
https://github.com/NixOS/nixpkgs/pull/244803

Solution which uses docker backend.
https://kressle.in/immich

Podman networking.
Note that it doesn't mention that DNS could be broken by firewall/interface rules.
https://github.com/containers/podman/blob/main/docs/tutorials/basic_networking.md

My original source.
https://gist.github.com/mfenniak/c6f6b1cde07fc33df4d925e13f7d5afa
