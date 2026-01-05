# Directive-97: Kernel Parameter Injection Guide

To achieve **Pure Silence** on the OODA Loop cores (4-15), we must instruct the Linux Kernel to ignore them at boot time.

## 1. Edit GRUB Configuration

Open `/etc/default/grub` with root privileges.

```bash
sudo nano /etc/default/grub
```

## 2. Inject Parameters

Find the line starting with `GRUB_CMDLINE_LINUX_DEFAULT` and append the following "Iron Floor" parameters:

```text
isolcpus=4-15 nohz_full=4-15 rcu_nocbs=4-15 processor.max_cstate=1 intel_idle.max_cstate=0 idle=poll
```

### Explanation

- `isolcpus=4-15`: Isolates cores from the general SMP scheduler. No user processes will run here unless explicitly pinned via `taskset`.
- `nohz_full=4-15`: Stops the scheduler tick (1000Hz/250Hz) on these cores if only one task is running. This is CRITICAL for sub-5us jitter.
- `rcu_nocbs=4-15`: Offloads RCU (Read-Copy-Update) callbacks to housekeeping cores (0-3).
- `processor.max_cstate=1`: Prevents deep sleep states.
- `idle=poll`: Forces the CPU to loop-wait instead of halting when idle (lowest latency wakeup).

## 3. Update GRUB

Apply the changes:

```bash
# Ubuntu/Debian
sudo update-grub

# RHEL/CentOS
sudo grub2-mkconfig -o /boot/grub2/grub.cfg
```

## 4. Reboot

Reboot the system to apply the Iron Floor.

```bash
sudo reboot
```
