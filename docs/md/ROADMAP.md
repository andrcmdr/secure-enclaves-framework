## Roadmap: what's in progress, in active development and upcoming features, release schedule

### WIP (already in development):

- **0.9.0** - cryptography stack, for buffer level SLC and content encryption.

- **0.10.0** - integration with KMS for key storage, probably TPM module usage for local key storage.

- **0.11.0** - Advanced logging and log tracing for Pipeline and proxies. Web-RA already included structural logging.
        This will improve logs aggregation, especially in enclave's production mode (without debug console),
        for remote debugging of enclaves, for use in monitoring and log aggregation systems,
        to understand exact places where issues/bugs appeared, to reveal and fix them fast.

### The following is a subject to change, i.e. the order of releases and version numbers.

### In design stage:

- **0.12.0** - enclave's init system update, rewritten in Rust.

- **0.13.0** - Enclaves engine service with web API (Docker API like),
         for enclaves provisioning (EC2 or other cloud instances configuration, including NUMA, Huge Memory Pages allocation, etc.),
         enclaves building (integration with current reproducible builds system, rbuilds build system), enclaves deployment, monitoring.
         Integration with AWS SQS/MQ for deployment tasks tracking and backend systems. Integration with CI (GitHub Actions mostly).

- **0.14.0** - port of framework to QEMU VM with EIF images support for running,
         along with qCoW images, block (disk) devices and PCI (PCI-E NVMe) devices attestation, FS monitoring and attestation for qCoW images.
