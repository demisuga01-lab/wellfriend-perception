//! Manual scalar restoration benchmark; CI only validates that it compiles.

use std::time::Instant;

use wellfriend_perception_core::{ImageBuffer, PixelFormat, benchmarks::BenchmarkRecord};
use wellfriend_perception_restoration::{
    ConditionVector, DeviceClass, DocumentFilterGraph, DocumentFilterPreset, SpecialistRouter,
};

fn main() {
    let image = ImageBuffer::new(
        128,
        96,
        PixelFormat::Gray8,
        (0..128 * 96)
            .map(|index| 112 + (index % 20) as u8)
            .collect(),
    )
    .expect("fixed synthetic benchmark fixture is valid");
    let started = Instant::now();
    for _ in 0..8 {
        let output = DocumentFilterGraph::default()
            .apply(
                DocumentFilterPreset::Clean,
                &image,
                &ConditionVector::default(),
                DeviceClass::Mid,
            )
            .expect("scalar clean filter must run on fixture");
        std::hint::black_box(output);
    }
    println!(
        "{}",
        BenchmarkRecord::synthetic_baseline(
            "document",
            "mp4-low-contrast-clean",
            "clean_filter",
            8,
            started.elapsed().as_nanos(),
        )
        .to_json_line()
    );
    for preset in [
        DocumentFilterPreset::Grayscale,
        DocumentFilterPreset::BlackAndWhite,
    ] {
        let started = Instant::now();
        let output = DocumentFilterGraph::default()
            .apply(
                preset,
                &image,
                &ConditionVector::default(),
                DeviceClass::Mid,
            )
            .expect("baseline filter must run");
        std::hint::black_box(output);
        println!(
            "{}",
            BenchmarkRecord::synthetic_baseline(
                "document",
                "mp4-filter-fixture",
                format!("{:?}_filter", preset),
                1,
                started.elapsed().as_nanos(),
            )
            .to_json_line()
        );
    }
    let started = Instant::now();
    let decision = SpecialistRouter::default()
        .plan(
            &ConditionVector::default(),
            DocumentFilterPreset::Clean,
            DeviceClass::Low,
        )
        .expect("empty condition plan must be valid");
    std::hint::black_box(decision);
    println!(
        "{}",
        BenchmarkRecord::synthetic_baseline(
            "document",
            "mp4-router-condition-plan",
            "condition_to_plan",
            1,
            started.elapsed().as_nanos(),
        )
        .to_json_line()
    );
}
