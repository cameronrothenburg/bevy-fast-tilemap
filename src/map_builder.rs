use super::prelude::*;
use bevy::{math::uvec2, prelude::*};

use super::tile_projection::TileProjection;

/// Builder for constructing a map component. This is usually the preferred way of constructing.
pub struct MapBuilder<C: Customization = NoCustomization> {
    map: Map<C>,
}

impl<C: Customization> MapBuilder<C> {
    /// Create a builder for the given map size (number of tiles in each dimension),
    /// the given atlas texture and the tile size (in the atlas).
    pub fn new(map_size: UVec2, atlas_texture: Handle<Image>, tile_size: Vec2) -> Self {
        Self {
            map: Map::<C> {
                atlas_texture,
                map_uniform: MapUniform {
                    map_size,
                    tile_size,
                    ..default()
                },
                perspective_overhangs: true,
                perspective_underhangs: true,
                dominance_overhangs: false,
                ..default()
            },
        }
    } // fn new

    /// Create a builder for the given map size (number of tiles in each dimension),
    /// the given atlas texture and the tile size (in the atlas).
    pub fn custom(
        map_size: UVec2,
        atlas_texture: Handle<Image>,
        tile_size: Vec2,
        user_data: C::UserData,
    ) -> Self {
        Self {
            map: Map::<C> {
                atlas_texture,
                map_uniform: MapUniform {
                    map_size,
                    tile_size,
                    ..default()
                },
                perspective_overhangs: true,
                perspective_underhangs: true,
                dominance_overhangs: false,
                user_data,
                ..default()
            },
        }
    } // fn new

    pub fn with_atlas_tile_size_factor(mut self, factor: i32) -> Self {
        self.map.map_uniform.atlas_tile_size_factor = factor;
        self
    }

    pub fn with_user_data(mut self, new_user_data: C::UserData) -> Self {
        self.map.user_data = new_user_data;
        self
    }

    /// Us the given map projection for rendering. Default is [`crate::tile_projection::IDENTITY`],
    /// which will render the tiles in rectangular layout.
    pub fn with_projection(mut self, projection: TileProjection) -> Self {
        self.map.map_uniform.projection = projection.projection;
        self.map.map_uniform.tile_anchor_point = projection.tile_anchor_point;
        self
    }

    /// None: Automatically calculate number of tiles in rows/columns of the atlas
    ///       from atlas image size and tile size.
    /// Some(UVec2): Force the number of tiles in rows/columns of the atlas.
    ///              This can be useful if you have a tile atlas with a lot of empty space
    ///              or one that doesn't line up with the tile size / padding for some reason.
    pub fn with_n_tiles(mut self, force_n_tiles: Option<UVec2>) -> Self {
        self.map.force_n_tiles = force_n_tiles;
        self
    }

    /// Specify the padding in the `atlas_texture`.
    /// `inner`: Padding between the tiles,
    /// `topleft`: Padding to top and left of the tile atlas,
    /// `bottomright`: Padding to bottom and right of the atlas.
    ///
    /// Note that it is crucial that these values are precisely correct,
    /// we use them internally to determine how many tiles there are in the atlas in each
    /// direction, if that does not produce a number close to an integer,
    /// you will get a `panic` when the tile atlas is loaded.
    pub fn with_padding(mut self, inner: Vec2, topleft: Vec2, bottomright: Vec2) -> Self {
        self.map.map_uniform.inner_padding = inner;
        self.map.map_uniform.outer_padding_topleft = topleft;
        self.map.map_uniform.outer_padding_bottomright = bottomright;
        self
    }

    /// Render this map in "dominance" overhang mode.
    /// "Dominance" overhang draws the overlap of tiles depending on their index in the tile atlas.
    /// Tiles with higher index will be drawn on top of tiles with lower index.
    /// For this we draw in the "padding" area of the tile atlas.
    pub fn with_dominance_overhang(mut self) -> Self {
        self.map.dominance_overhangs = true;
        self.map.perspective_overhangs = false;
        self.map.perspective_underhangs = false;
        self
    }

    /// Render this map in "perspective" overhang mode.
    /// "Perspective" overhang draws the overlap of tiles depending on their "depth" that is the
    /// y-axis of their world position (tiles higher up are considered further away).
    pub fn with_perspective_overhang(mut self) -> Self {
        self.map.dominance_overhangs = false;
        self.map.perspective_overhangs = true;
        self.map.perspective_underhangs = true;
        self
    }

    /// Specify directions (eg vec2(-1.0, 1.0)) for underhangs.
    ///
    /// Use this to manually specify the *under*hang directions you want to use
    /// (overhangs are implicitly the opposite direction).
    /// This can be useful if you are using IDENTITY projection but still want some
    /// over/underhangs other than dominance.
    pub fn with_forced_underhangs(mut self, underhangs: Vec<Vec2>) -> Self {
        self.map.dominance_overhangs = false;
        self.map.perspective_underhangs = true;
        self.map.perspective_overhangs = true;
        self.map.force_underhangs = underhangs;
        self
    }

    pub fn with_overhangs(
        mut self,
        dominance: bool,
        perspective_under: bool,
        perspective_over: bool,
    ) -> Self {
        self.map.dominance_overhangs = dominance;
        self.map.perspective_underhangs = perspective_under;
        self.map.perspective_overhangs = perspective_over;
        self
    }

    /// Build the map component.
    pub fn build(self) -> Map<C> {
        self.build_and_initialize(|_| {})
    }

    /// Build the map component and immediately initialize the map
    /// data with the given initializer callback.
    /// The callback will receive a mutable reference to a `MapIndexer`.
    pub fn build_and_initialize<F>(mut self, initializer: F) -> Map<C>
    where
        F: FnOnce(&mut MapIndexerMut<C>),
    {
        self.map.map_texture.resize(
            (self.map.map_size().x * self.map.map_size().y) as usize,
            0u32,
        );

        initializer(&mut MapIndexerMut::<C> { map: &mut self.map });

        self.map.update_inverse_projection();
        self.map.map_uniform.update_world_size();

        self.map
    } // fn build_and_initialize

    /// Build the map component and immediately initialize the map
    /// data with the given initializer callback.
    /// The callback will receive a `UVec2` and return a `u32`.
    pub fn build_and_set<F>(self, mut initializer: F) -> Map<C>
    where
        F: FnMut(UVec2) -> u32,
    {
        let sx = self.map.map_size().x;
        let sy = self.map.map_size().y;

        self.build_and_initialize(|m: &mut MapIndexerMut<C>| {
            for y in 0..sy {
                for x in 0..sx {
                    m.set(x, y, initializer(uvec2(x, y)));
                }
            }
        })
    } // build_and_set()
}
