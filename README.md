# KPL (kube-podlog)

kube-podlog is an early-stage Rust project exploring a faster, more reliable
alternative to `kubectl logs -f` for streaming logs from multiple Kubernetes
pods.

The goal of this project is to build a production-quality CLI tool that can:
- Stream logs from multiple pods concurrently
- Follow pods across restarts and rollouts
- Merge log output into a single stream
- Behave predictably under load (SRE-focused design)

This repository is under active development and currently contains only
the initial project scaffolding (CLI, config, and error handling).

More functionality will be added incrementally.

## Status
Work in progress
Not ready for use yet.
