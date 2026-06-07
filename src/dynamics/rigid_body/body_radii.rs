//! Radii associated with a [`RigidBody`] and its colliders.
//!
//! See [`BodyRadii`] for more information.

use core::marker::PhantomData;

use bevy::{ecs::system::StaticSystemParam, prelude::*};

use crate::{
    collision::collider::{
        AnyCollider, ColliderContext, collider_hierarchy::RigidBodyColliders,
        collider_transform::ColliderTransform,
    },
    dynamics::rigid_body::mass_properties::components::ComputedCenterOfMass,
    math::Scalar,
    schedule::{PhysicsSchedule, PhysicsStepSystems},
};

/// Radii associated with a [`RigidBody`] and its colliders.
///
/// These can be used for various purposes, such as determining thresholds
/// for Continuous Collision Detection (CCD), sleeping, and contact recycling.
///
/// The values are automatically computed and updated by the [`BodyRadiiPlugin`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct BodyRadii {
    /// The minimum "core" thickness of the colliders attached to the body.
    ///
    /// Typically corresponds to the minimum distance from the centroid
    /// of any given shape to its surface.
    ///
    /// This can be useful for Continuous Collision Detection (CCD)
    /// to determine how far the body can move before it might start to tunnel
    /// through other geometry, for example.
    pub min_thickness: Scalar,

    /// The maximum distance from the center of mass of the body to the surface
    /// of any of its colliders.
    ///
    /// Typically corresponds to the radius of the sphere formed by sweeping
    /// the body about its center of mass.
    ///
    /// This can be useful for determining the maximum velocity that a point
    /// on the body can have, which can be used for sleeping and contact recycling
    /// thresholds, for example.
    pub sweep_radius: Scalar,
}

impl Default for BodyRadii {
    fn default() -> Self {
        Self {
            min_thickness: Scalar::INFINITY,
            sweep_radius: 0.0,
        }
    }
}

/// A plugin for computing and updating [`BodyRadii`] for rigid bodies.
#[derive(Default)]
pub struct BodyRadiiPlugin<C: AnyCollider> {
    _phantom: PhantomData<C>,
}

impl<C: AnyCollider> Plugin for BodyRadiiPlugin<C> {
    fn build(&self, app: &mut App) {
        // Update body radii before they are consumed by the solver and continuous collision
        // detection. Allowing ambiguities lets multiple collision backends coexist.
        app.add_systems(
            PhysicsSchedule,
            update_body_radii::<C>
                .before(PhysicsStepSystems::Solver)
                .ambiguous_with_all(),
        );
    }
}

fn update_body_radii<C: AnyCollider>(
    mut bodies: Query<(
        &mut BodyRadii,
        &RigidBodyColliders,
        Ref<ComputedCenterOfMass>,
    )>,
    colliders: Query<(Entity, &C, &ColliderTransform)>,
    changed_colliders: Query<(), Or<(Changed<C>, Changed<ColliderTransform>)>>,
    context: StaticSystemParam<C::Context>,
) {
    let context = context.into_inner();

    for (mut body_radii, rb_colliders, com) in bodies.iter_mut() {
        if !com.is_changed()
            && !rb_colliders
                .iter()
                .any(|collider_entity| changed_colliders.contains(collider_entity))
        {
            // Neither the center of mass nor any of the colliders have changed.
            continue;
        }

        let mut min_thickness: Scalar = Scalar::INFINITY;
        let mut sweep_radius: Scalar = 0.0;

        // Find the minimum thickness and maximum sweep radius.
        for (entity, collider, collider_transform) in colliders.iter_many(rb_colliders) {
            // Compute the minimum thickness
            let ctx = ColliderContext::new(entity, &context);
            let thickness = collider.min_thickness_with_context(ctx);
            min_thickness = min_thickness.min(thickness);

            // Compute the sweep radius
            let ctx = ColliderContext::new(entity, &context);
            let point = com.0 - collider_transform.translation;
            let distance_to_com = collider.max_distance_to_point_with_context(point, ctx);
            sweep_radius = sweep_radius.max(distance_to_com);
        }

        body_radii.min_thickness = min_thickness;
        body_radii.sweep_radius = sweep_radius;
    }
}
