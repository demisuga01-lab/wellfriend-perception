# Model artifact contract

Runtime model artifacts are produced by `wellfriend-models`, never by importing Python training code. A release directory contains `model.onnx`, `manifest.json`, `preprocess.json`, `postprocess.json`, `labels.json`, `checksums.json`, `metrics.json`, `LICENSES/`, and `README.md`. The manifest must state model/version/task/domain/input shape and dynamic-shape support/pixel format/preprocessing/postprocessing/output schema/dataset references/license notes/intended runtime/device class/metrics/hashes. Runtime adapters reject artifacts that fail checksum or schema validation.

