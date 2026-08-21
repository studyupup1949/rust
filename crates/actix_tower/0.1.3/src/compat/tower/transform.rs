//! Bidirectional conversion between Actix Transforms and Tower Layers.

use actix_service::Transform as ActixTransform;
use actix_web::dev::ServiceRequest;
use tower_layer::Layer as TowerLayerTrait;

use crate::compat::tower::layer::TowerLayer;

/// Trait extension to convert an Actix `Transform` into a Tower `Layer`.
///
/// This is useful when you want to use an Actix middleware inside a
/// Tower service stack.
pub trait TransformAsLayer {
    /// Convert this Actix transform into a Tower layer.
    fn into_layer<S>(self) -> impl TowerLayerTrait<S>
    where
        S: Clone;
}

/// Trait extension to convert a Tower `Layer` into an Actix `Transform`.
///
/// This is the primary mechanism for using Tower middleware with Actix.
pub trait LayerAsTransform {
    /// Convert this Tower layer into an Actix transform.
    fn into_transform<S>(self) -> TowerLayer<Self>
    where
        Self: Sized,
    {
        TowerLayer::new(self)
    }
}

// Blanket impl: any Tower Layer can become an Actix Transform
impl<L> LayerAsTransform for L where L: TowerLayerTrait<()> {}

// Note: Converting an Actix Transform to a Tower Layer is more complex
// because it requires running the Actix runtime. This is provided as a
// stub for future implementation.
impl<T> TransformAsLayer for T
where
    T: ActixTransform<(), ServiceRequest>,
{
    fn into_layer<S>(self) -> impl TowerLayerTrait<S>
    where
        S: Clone,
    {
        // This is a placeholder. Full implementation would require
        // wrapping the Actix transform in a Tower-compatible layer.
        tower_layer::layer_fn(move |service: S| service)
    }
}
