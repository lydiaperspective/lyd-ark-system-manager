# Lyd Ark System Manager

The Lyd Ark System Manager is the management backend for Lyd Ark systems. It starts as a Rust CLI and is designed to provide a stable foundation for a future KDE graphical interface.

## Current status

The initial CLI foundation provides read-only system information, standardized storage-location discovery, and configuration inspection. It does not yet move data, change mount points, manage containers, or enable network services.

## Standardized storage root

The default root is `/ark`:

| Logical location | Default path | Intended use |
|---|---|---|
| Containers | `/ark/Containers` | Compose projects, container project files, and managed application data |
| Media | `/ark/Media` | Media libraries |
| Share | `/ark/Share` | Files shared over NFS or Samba |
| Configs | `/ark/Configs` | User-managed service and application configuration |

These are stable paths for applications and Compose files. Their physical storage destinations will become configurable in a later storage-management phase.

**Important:** Podman's internal graph root is not the same thing as the `Containers` folder. Relocating Podman storage requires a separate, Podman-aware migration and is intentionally not part of the initial CLI.

## Build

Requires a current stable Rust toolchain.

```bash
cargo build --release
cargo test
```

The binary is named `ark-system-manager`.

## Usage

```bash
ark-system-manager --help
ark-system-manager info
ark-system-manager config path
ark-system-manager config show
ark-system-manager storage list
ark-system-manager storage status
```

The default configuration file is:

```text
/etc/ark/system-manager.toml
```

If it does not exist, the CLI uses built-in defaults. For development and testing, set `ARK_SYSTEM_MANAGER_CONFIG` to an alternate TOML file.

See [docs/CONFIGURATION.md](docs/CONFIGURATION.md) for the configuration format.

## Roadmap

- [x] Rust CLI project foundation
- [x] Standardized `/ark` storage layout defaults
- [x] Read-only storage and configuration inspection
- [ ] Persistent storage-location configuration
- [ ] Storage destination validation
- [ ] Safe storage migration with verification and rollback
- [ ] Managed bind mounts for stable application paths
- [ ] Container/service dependency discovery and safe stop/start orchestration
- [ ] NFS server integration
- [ ] Samba server integration
- [ ] Server-image packaging and integration
- [ ] Shared backend interface for a future KDE GUI

## Safety principles

- Never silently overwrite or delete user data.
- Never switch a storage location before the destination is validated and data is verified.
- Keep the previous location available until a migration is committed.
- Treat missing external disks as unavailable; never create a normal directory underneath an expected mount point and mistake it for the mounted storage.
- Do not automatically expose NFS or Samba shares to untrusted networks.
- Keep privileged operations explicit, auditable, and narrowly scoped.
