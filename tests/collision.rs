use bevy::prelude::*;
use spacetrash_v11::components::{WallSegment, WallSegmentCollider};
use spacetrash_v11::procgen::tile::WallDirection;
use spacetrash_v11::systems::collision::{
    resolve_circle_pair, resolve_circle_vs_aabb, resolve_circle_vs_wall_segment,
};

fn assert_vec_eq(actual: Vec3, expected: Vec3) {
    assert!(
        (actual - expected).length() < 1e-5,
        "expected {expected:?}, got {actual:?}"
    );
}

fn wall_segment_world(seg: &WallSegment) -> (Vec3, Vec3, Vec3) {
    (
        Vec3::new(seg.a.x, 0.0, seg.a.y),
        Vec3::new(seg.b.x, 0.0, seg.b.y),
        Vec3::new(seg.normal.x, 0.0, seg.normal.y),
    )
}

#[test]
fn no_collision_when_circle_is_far_from_aabb() {
    let resolved = resolve_circle_vs_aabb(
        Vec3::new(0.0, 0.25, 0.0),
        0.1667,
        Vec3::new(3.0, 0.5, 0.0),
        Vec3::new(1.0, 1.0, 0.5),
    );

    assert_vec_eq(resolved, Vec3::new(0.0, 0.25, 0.0));
}

#[test]
fn touching_aabb_edge_is_unchanged() {
    let resolved = resolve_circle_vs_aabb(
        Vec3::new(-1.1667, 0.25, 0.0),
        0.1667,
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(1.0, 1.0, 0.5),
    );

    assert_vec_eq(resolved, Vec3::new(-1.1667, 0.25, 0.0));
}

#[test]
fn circle_pushes_out_of_aabb_from_right_side() {
    let resolved = resolve_circle_vs_aabb(
        Vec3::new(1.05, 0.25, 0.0),
        0.1667,
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(1.0, 1.0, 0.5),
    );

    assert_vec_eq(resolved, Vec3::new(1.1667, 0.25, 0.0));
}

#[test]
fn circle_pushes_out_of_aabb_from_left_side() {
    let resolved = resolve_circle_vs_aabb(
        Vec3::new(-1.05, 0.25, 0.0),
        0.1667,
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(1.0, 1.0, 0.5),
    );

    assert_vec_eq(resolved, Vec3::new(-1.1667, 0.25, 0.0));
}

#[test]
fn circle_center_inside_aabb_pushes_out_along_minimal_axis() {
    let resolved = resolve_circle_vs_aabb(
        Vec3::new(0.2, 0.25, 0.1),
        0.1667,
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(1.0, 1.0, 0.5),
    );

    assert_vec_eq(resolved, Vec3::new(0.2, 0.25, 0.5));
}

#[test]
fn circle_center_inside_aabb_pushes_out_along_x_when_smaller() {
    let resolved = resolve_circle_vs_aabb(
        Vec3::new(0.8, 0.25, 0.1),
        0.1667,
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(1.0, 1.0, 0.5),
    );

    assert_vec_eq(resolved, Vec3::new(1.0, 0.25, 0.1));
}

#[test]
fn circle_pair_no_collision_when_far_apart() {
    let (a, b) = resolve_circle_pair(
        Vec3::new(0.0, 0.25, 0.0),
        0.1667,
        Vec3::new(3.0, 0.25, 0.0),
        0.1667,
    );

    assert_vec_eq(a, Vec3::new(0.0, 0.25, 0.0));
    assert_vec_eq(b, Vec3::new(3.0, 0.25, 0.0));
}

#[test]
fn circle_pair_pushes_both_apart_symmetrically() {
    let (a, b) = resolve_circle_pair(
        Vec3::new(0.3, 0.25, 0.0),
        0.1667,
        Vec3::new(0.0, 0.25, 0.0),
        0.1667,
    );

    let min_dist = 0.1667 + 0.1667;
    let half = min_dist / 2.0;
    let midpoint = (0.3 + 0.0) * 0.5;
    assert_vec_eq(a, Vec3::new(midpoint + half, 0.25, 0.0));
    assert_vec_eq(b, Vec3::new(midpoint - half, 0.25, 0.0));
}

#[test]
fn circle_pair_identical_centers_separates_along_x() {
    let (a, b) = resolve_circle_pair(
        Vec3::new(0.0, 0.25, 0.0),
        0.1667,
        Vec3::new(0.0, 0.25, 0.0),
        0.1667,
    );

    let min_dist = 0.1667 + 0.1667;
    let half = min_dist / 2.0;
    assert_vec_eq(a, Vec3::new(half, 0.25, 0.0));
    assert_vec_eq(b, Vec3::new(-half, 0.25, 0.0));
}

#[test]
fn north_wall_pushes_actor_southward() {
    let collider = WallSegmentCollider::for_direction(WallDirection::North);
    let seg = &collider.segments[0];
    let (a, b, normal) = wall_segment_world(seg);

    let resolved = resolve_circle_vs_wall_segment(
        Vec3::new(0.0, 0.0, -0.6),
        0.2,
        a,
        b,
        normal,
    );

    assert!(
        resolved.z < -0.6,
        "north wall should push actor further south, got {resolved:?}"
    );
}

#[test]
fn south_wall_pushes_actor_northward() {
    let collider = WallSegmentCollider::for_direction(WallDirection::South);
    let seg = &collider.segments[0];
    let (a, b, normal) = wall_segment_world(seg);

    let resolved = resolve_circle_vs_wall_segment(
        Vec3::new(0.0, 0.0, 0.6),
        0.2,
        a,
        b,
        normal,
    );

    assert!(
        resolved.z > 0.6,
        "south wall should push actor further north, got {resolved:?}"
    );
}

#[test]
fn east_wall_pushes_actor_westward() {
    let collider = WallSegmentCollider::for_direction(WallDirection::East);
    let seg = &collider.segments[0];
    let (a, b, normal) = wall_segment_world(seg);

    let resolved = resolve_circle_vs_wall_segment(
        Vec3::new(-0.6, 0.0, 0.0),
        0.2,
        a,
        b,
        normal,
    );

    assert!(
        resolved.x < -0.6,
        "east wall should push actor further west, got {resolved:?}"
    );
}

#[test]
fn west_wall_pushes_actor_eastward() {
    let collider = WallSegmentCollider::for_direction(WallDirection::West);
    let seg = &collider.segments[0];
    let (a, b, normal) = wall_segment_world(seg);

    let resolved = resolve_circle_vs_wall_segment(
        Vec3::new(0.6, 0.0, 0.0),
        0.2,
        a,
        b,
        normal,
    );

    assert!(
        resolved.x > 0.6,
        "west wall should push actor further east, got {resolved:?}"
    );
}

#[test]
fn corner_wall_has_two_inward_segments() {
    let ne = WallSegmentCollider::for_direction(WallDirection::NorthEast);
    assert_eq!(ne.segments.len(), 2);

    let nw = WallSegmentCollider::for_direction(WallDirection::NorthWest);
    assert_eq!(nw.segments.len(), 2);

    let se = WallSegmentCollider::for_direction(WallDirection::SouthEast);
    assert_eq!(se.segments.len(), 2);

    let sw = WallSegmentCollider::for_direction(WallDirection::SouthWest);
    assert_eq!(sw.segments.len(), 2);
}
