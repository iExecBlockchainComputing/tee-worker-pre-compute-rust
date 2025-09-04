# Changelog

## 0.1.0 (2025-09-04)


### Features

* Add app_runner crate ([#8](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/8)) ([18a091c](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/18a091c36ed7f66506718ba847cd5a20b9b6d3b8))
* add ExitMode enum for process exit handling ([#25](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/25)) ([e77c0d7](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/e77c0d7431274dbe5f3625589f8b94aefb667a9b))
* Add hash_utils crate ([#4](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/4)) ([0399085](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/0399085b6afd4c69e7c407e4c2cb360abb041ebf))
* add input file download ([#13](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/13)) ([97888e3](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/97888e33b7acf0d0a27c6826172593a88a1d0564))
* Add signer crate ([#6](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/6)) ([048fb52](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/048fb52aa15d030db96a12eb9119cc51fde59b51))
* implement dataset download and decryption ([#17](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/17)) ([1b9f18f](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/1b9f18f9ac8a3903252730367d12a8cefcc04844))
* implement environment arguments handling ([#11](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/11)) ([0cf763a](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/0cf763a887e0bc94ef917c401b7981ad9b63ea3f))
* improve exit cause reporting with detailed logs and tests ([#21](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/21)) ([0ff2061](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/0ff206103ad692324661f03e513b75fabf371792))
* improve logger initialization to set a default log level ([#20](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/20)) ([9f5a45c](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/9f5a45cc7c7ec2375e5d4f867950d35c623aa13e))
* initialize env_logger in main for better runtime debugging ([#18](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/18)) ([d047dd6](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/d047dd6047edb997ec998699a6f55b9086f66363))
* replace OpenSSL AES-CBC with Rust Crypto aes + cbc crates ([#23](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/23)) ([f555579](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/f5555796273f6dd1623723e75c83e1e0fdbfbb1c))


### Bug Fixes

* align Rust environment variable names with Java equivalents ([#19](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/19)) ([5497e5d](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/5497e5d1305d9e383d6016a5a5b77d7829ea0fa0))
* redirect log output from stderr to stdout ([#22](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/22)) ([d2a2ae9](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/d2a2ae9123003e830d59f456f8afc49d390e46cc))
* Remove PreComputeError wrapper ([#9](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/issues/9)) ([5988e80](https://github.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/commit/5988e80c8835e734749a55f7a5481c7c99feac34))
