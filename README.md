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

The executable command is `lydsys`.

## Usage

```bash
lydsys --help
lydsys info
lydsys config path
lydsys config show
lydsys storage list
lydsys storage status
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

## Permissions and rootless containers

On Lyd Ark images, the standardized `/ark` tree is intended to be accessible to the administrator without sudo. The image creates an `ark` group and gives the initial administrator membership in that group. The standardized directories use group ownership `ark` and setgid permissions so newly created entries inherit the shared group.

Rootless Podman runs as the administrator's UID. Bind-mounted files still obey normal host UID/GID and permission checks; group membership on the host does not automatically guarantee that every container process receives supplementary groups. For group-based access inside a container, configure Podman to pass the relevant group (for example, `--group-add keep-groups` when supported by the selected OCI runtime) or use an explicit mapped GID and suitable directory permissions. Do not make all storage world-writable as a workaround.

GPU device nodes are under `/dev/dri` (not `/dri`). The initial administrator is added to the `video` and `render` groups so it can access DRM devices when present. A container must also be given the device, such as `--device /dev/dri`, and retain the required group permissions. Actual GPU acceleration additionally depends on host drivers and compatible userspace libraries in the container.

The image-level group and directory setup currently applies to newly installed systems and the first administrator account. Existing installations need a deliberate migration to create the groups, apply directory ownership, and add the administrator to supplementary groups; changing group membership takes effect after a new login session.
