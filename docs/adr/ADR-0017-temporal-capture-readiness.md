# ADR-0017: Temporal capture-readiness policy

## Status

Accepted — MP3.

## Decision

MP3 tracks and exponentially smooths accepted quads, derives position/scale/rotation stability, handles lost tracks, and combines that evidence with quality, coverage, fusion, and refinement for machine-readable readiness. It does not control a camera or render UI text.

## Consequences

Android, web, and desktop controllers can use one deterministic readiness policy while maintaining platform ownership. Optical flow, Kalman filtering, IMU integration, and device-specific auto-capture timing remain pluggable follow-up work.
