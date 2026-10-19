use bevy::prelude::*;

/// Resolves a circle collider against a static AABB collider.
///
/// Returns the corrected circle center position.
/// If there is no overlap, the original position is returned unchanged.
pub fn resolve_circle_vs_aabb(
    circle_pos: Vec3,
    radius: f32,
    collider_pos: Vec3,
    half_extents: Vec3,
) -> Vec3 {
    let min = collider_pos - half_extents;
    let max = collider_pos + half_extents;

    let closest = Vec3::new(
        circle_pos.x.clamp(min.x, max.x),
        0.0,
        circle_pos.z.clamp(min.z, max.z),
    );

    let delta = Vec3::new(
        circle_pos.x - closest.x,
        0.0,
        circle_pos.z - closest.z,
    );

    let dist_sq = delta.length_squared();
    let radius_sq = radius * radius;

    if dist_sq >= radius_sq {
        return circle_pos;
    }

    let dist = dist_sq.sqrt();

    if dist > 0.0 {
        let penetration = radius - dist;
        let normal = delta / dist;
        return circle_pos + normal * penetration;
    }

    // Circle center is inside the AABB. Push out along the minimal axis.
    let center_delta = circle_pos - collider_pos;
    let overlap_x = half_extents.x - center_delta.x.abs();
    let overlap_z = half_extents.z - center_delta.z.abs();

    let mut corrected = circle_pos;

    if overlap_x < overlap_z {
        let sign = if center_delta.x > 0.0 { 1.0 } else { -1.0 };
        corrected.x += sign * overlap_x;
    } else {
        let sign = if center_delta.z > 0.0 { 1.0 } else { -1.0 };
        corrected.z += sign * overlap_z;
    }

    corrected
}

/// Resolves a dynamic circle against another circle by pushing only the first circle.
///
/// This is useful when one circle is considered static (e.g., a non-moving collider).
/// If the two centers are identical, the first circle is pushed along +X.
pub fn resolve_circle_vs_circle(
    circle_pos: Vec3,
    radius: f32,
    other_pos: Vec3,
    other_radius: f32,
) -> Vec3 {
    let delta = circle_pos - other_pos;
    let dist_sq = delta.length_squared();
    let min_dist = radius + other_radius;

    if dist_sq >= min_dist * min_dist {
        return circle_pos;
    }

    if dist_sq > 0.0 {
        let dist = dist_sq.sqrt();
        let normal = delta / dist;
        let penetration = min_dist - dist;
        circle_pos + normal * penetration
    } else {
        // Identical centers: push along +X deterministically.
        circle_pos + Vec3::X * min_dist
    }
}

/// Resolves overlap between two dynamic circles by moving both equally.
///
/// Returns corrected positions for both circles.
pub fn resolve_circle_pair(
    pos_a: Vec3,
    radius_a: f32,
    pos_b: Vec3,
    radius_b: f32,
) -> (Vec3, Vec3) {
    let delta = pos_a - pos_b;
    let dist_sq = delta.length_squared();
    let min_dist = radius_a + radius_b;

    if dist_sq >= min_dist * min_dist {
        return (pos_a, pos_b);
    }

    if dist_sq > 0.0 {
        let dist = dist_sq.sqrt();
        let normal = delta / dist;
        let penetration = min_dist - dist;
        let half = penetration * 0.5;
        (pos_a + normal * half, pos_b - normal * half)
    } else {
        // Identical centers: separate along +X deterministically.
        let half = min_dist * 0.5;
        (pos_a + Vec3::X * half, pos_b - Vec3::X * half)
    }
}

/// Finds the closest point on segment `a`..`b` to `point`.
fn closest_point_on_segment(point: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let ab = b - a;
    let ab_len_sq = ab.length_squared();

    if ab_len_sq <= f32::EPSILON {
        return a;
    }

    let ap = point - a;
    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    a + ab * t
}

/// Resolves a circle against a single inward-facing wall segment.
///
/// `normal` must point from the wall into the room. When the circle overlaps
/// the segment, it is pushed back along that normal.
pub fn resolve_circle_vs_wall_segment(
    circle_pos: Vec3,
    radius: f32,
    a: Vec3,
    b: Vec3,
    normal: Vec3,
) -> Vec3 {
    let normal = normal.normalize_or_zero();
    if normal == Vec3::ZERO {
        return circle_pos;
    }

    let closest = closest_point_on_segment(circle_pos, a, b);
    let dist_sq = (circle_pos - closest).length_squared();

    if dist_sq >= radius * radius {
        return circle_pos;
    }

    let dist = dist_sq.sqrt();

    if dist > 0.0 {
        circle_pos + normal * (radius - dist)
    } else {
        circle_pos + normal * radius
    }
}
