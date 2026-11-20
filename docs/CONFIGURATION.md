# Configuration

The System Manager reads TOML configuration from:

```text
/etc/ark/system-manager.toml
```

Set `ARK_SYSTEM_MANAGER_CONFIG` to use a different file for development or testing.

If the file is absent, built-in defaults are used. A present but invalid file is an error; the CLI will not silently ignore malformed configuration.

## Initial schema

```toml
[storage]
root = "/ark"

[storage.locations]
Containers = "/ark/Containers"
Media = "/ark/Media"
Share = "/ark/Share"
Configs = "/ark/Configs"
```

All configured paths must be absolute. The initial release only reads this configuration. It does not create directories, change mounts, or move data.

## Storage path semantics

The logical locations are stable application-facing paths. In a later phase, each logical location will have a configurable physical destination, managed through mount units (preferably bind mounts) so applications continue using the same logical path.

The manager must distinguish an unavailable mount from an ordinary directory on the root filesystem. It must never write into an unmounted external-disk mount point as a fallback.
