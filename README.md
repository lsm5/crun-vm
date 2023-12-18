# The `crun-qemu` OCI runtime

This is an **experimental** [OCI Runtime] that enables `podman run` to work with
VM images. The objective is to make running VMs (in simple configurations) as
easy as running containers.

## Trying it out

First build the runtime:

```console
$ dnf install bash coreutils crun genisoimage libvirt-client libvirt-daemon-driver-qemu libvirt-daemon-log qemu-img shadow-utils util-linux virtiofsd
$ cargo build
```

Then obtain a QEMU-compatible VM image and place it in a directory by itself:

```console
$ mkdir my-vm-image
$ curl -LO --output-dir my-vm-image https://download.fedoraproject.org/pub/fedora/linux/releases/39/Cloud/x86_64/images/Fedora-Cloud-Base-39-1.5.x86_64.qcow2
```

And try it out:

```console
$ podman run \
    --runtime "$PWD"/target/debug/crun-qemu \
    --security-opt label=disable \
    -it --rm \
    --rootfs my-vm-image \
    ""
```

The VM console should take over your terminal. To abort the VM, press `ctrl-]`.

You can also detach from the VM without terminating it by pressing `ctrl-p,
ctrl-q`. Afterwards, reattach by running:

```console
$ podman attach --latest
```

This command also works when you start the VM in detached mode using
podman-run's `-d`/`--detach` flags.

It's also possible to omit flags `-i`/`--interactive` and `-t`/`-tty` to
podman-run, in which case you won't be able to interact with the VM but can
still observe its console. Note that pressing `ctrl-]` will have no effect, so
use `podman container rm --force --time=0 ...` to terminate the VM instead.

## Using containerized VM images

This runtime also works with container images that contain a VM image file with
any name under `/` or under `/disk/`. No other files may exist in those
directories. Containers built for use as [KubeVirt `containerDisk`s] follow this
convention, so you can use those here:

```console
$ podman run \
    --runtime "$PWD"/target/debug/crun-qemu \
    --security-opt label=disable \
    -it --rm \
    quay.io/containerdisks/fedora:39 \
    ""
```

You can also use `util/package-vm-image.sh` to easily package a VM image into a
container image, and `util/extract-vm-image.sh` to extract a VM image contained
in a container image.

## Bind mounts

Bind mounts are passed through to the VM as [virtiofs] file systems:

```console
$ podman run \
    --runtime "$PWD"/target/debug/crun-qemu \
    --security-opt label=disable \
    -it --rm \
    -v ./util:/home/fedora/util \
    quay.io/containerdisks/fedora:39 \
    ""
```

If the VM image support cloud-init, the volume will automatically be mounted
inside the guest at the given path. Otherwise, you can mount it with:

```console
mount -t virtiofs /home/fedora/util /home/fedora/util
```

## cloud-init

You can provide a [cloud-init] NoCloud configuration to the VM by configuring a
bind mount with the special destination `/cloud-init`:

```console
$ ls examples/cloud-init/config/
meta-data  user-data  vendor-data

$ podman run \
    --runtime "$PWD"/target/debug/crun-qemu \
    --security-opt label=disable \
    -it --rm \
    quay.io/containerdisks/fedora:39 \
    --cloud-init examples/cloud-init/config
```

You should now be able to login with the default `fedora` username and password
`pass`.

## Ignition

Similarly, you can provide an [Ignition] configuration to the VM by configuring
a bind mount with the special destination `/ignition`:

```console
$ podman run \
    --runtime "$PWD"/target/debug/crun-qemu \
    --security-opt label=disable \
    -it --rm \
    quay.io/crun-qemu/fedora-coreos:39 \
    --ignition examples/ignition/config.ign
```

You should now be able to login with the default `core` username and password
`pass`.

## SSH'ing into the VM

Assuming the VM supports cloud-init, you can SSH into it using podman-exec
as whatever user cloud-init considers to be the default for your VM image:

```console
$ podman exec --latest fedora
```

The last argument above, which would typically be the command name, determines
instead the name of the user to ssh into. A command can optionally be specified
with further arguments. If no command is specified, a login shell is initiated.
Note that in the latter case, you probably want to pass flags `-it` to
podman-exec.

If you actually just want to exec into the container in which the VM is running
(probably to debug some problem with `crun-qemu` itself), pass in `-` as the
username.

## Passing block devices through to the VM

If cloud-init is available, it is possible to pass block devices through to the
VM at a specific path using podman-run's `--device` flag:

```console
$ podman run \
    --runtime "$PWD"/target/debug/crun-qemu \
    --security-opt label=disable \
    -it --rm \
    --device /dev/ram0:/path/in/vm/my-disk \
    quay.io/containerdisks/fedora:39 \
    ""
```

You can also pass them in as bind mounts using the `-v`/`--volume` or `--mount`
flags.

## How it works

Internally, the `crun-qemu` runtime uses [crun] to run a different container
that in turn uses [libvirt] to run a [QEMU] guest using the user-specified VM
image.

## License

This project is released under the GPL 2.0 (or later) license. See
[LICENSE](LICENSE).

[cloud-init]: https://cloud-init.io/
[crun]: https://github.com/containers/crun
[KubeVirt `containerDisk`s]: https://kubevirt.io/user-guide/virtual_machines/disks_and_volumes/#containerdisk
[libvirt]: https://libvirt.org/
[Ignition]: https://coreos.github.io/ignition/
[OCI Runtime]: https://github.com/opencontainers/runtime-spec/blob/v1.1.0/spec.md
[QEMU]: https://www.qemu.org/
[virtiofs]: https://virtio-fs.gitlab.io/
