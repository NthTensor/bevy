use bevy_app::{Plugin, PluginGroup, PluginSet};

/// This plugin group will add all the default plugins for a *Bevy* application:
/// * [`LogPlugin`](crate::log::LogPlugin)
/// * [`TaskPoolPlugin`](crate::core::TaskPoolPlugin)
/// * [`TypeRegistrationPlugin`](crate::core::TypeRegistrationPlugin)
/// * [`FrameCountPlugin`](crate::core::FrameCountPlugin)
/// * [`TimePlugin`](crate::time::TimePlugin)
/// * [`TransformPlugin`](crate::transform::TransformPlugin)
/// * [`HierarchyPlugin`](crate::hierarchy::HierarchyPlugin)
/// * [`DiagnosticsPlugin`](crate::diagnostic::DiagnosticsPlugin)
/// * [`InputPlugin`](crate::input::InputPlugin)
/// * [`WindowPlugin`](crate::window::WindowPlugin)
/// * [`AssetPlugin`](crate::asset::AssetPlugin) - with feature `bevy_asset`
/// * [`ScenePlugin`](crate::scene::ScenePlugin) - with feature `bevy_scene`
/// * [`WinitPlugin`](crate::winit::WinitPlugin) - with feature `bevy_winit`
/// * [`RenderPlugin`](crate::render::RenderPlugin) - with feature `bevy_render`
/// * [`ImagePlugin`](crate::render::texture::ImagePlugin) - with feature `bevy_render`
/// * [`PipelinedRenderingPlugin`](crate::render::pipelined_rendering::PipelinedRenderingPlugin) - with feature `bevy_render` when not targeting `wasm32`
/// * [`CorePipelinePlugin`](crate::core_pipeline::CorePipelinePlugin) - with feature `bevy_core_pipeline`
/// * [`SpritePlugin`](crate::sprite::SpritePlugin) - with feature `bevy_sprite`
/// * [`TextPlugin`](crate::text::TextPlugin) - with feature `bevy_text`
/// * [`UiPlugin`](crate::ui::UiPlugin) - with feature `bevy_ui`
/// * [`PbrPlugin`](crate::pbr::PbrPlugin) - with feature `bevy_pbr`
/// * [`GltfPlugin`](crate::gltf::GltfPlugin) - with feature `bevy_gltf`
/// * [`AudioPlugin`](crate::audio::AudioPlugin) - with feature `bevy_audio`
/// * [`GilrsPlugin`](crate::gilrs::GilrsPlugin) - with feature `bevy_gilrs`
/// * [`AnimationPlugin`](crate::animation::AnimationPlugin) - with feature `bevy_animation`
///
/// [`DefaultPlugins`] obeys *Cargo* *feature* flags. Users may exert control over this plugin group
/// by disabling `default-features` in their `Cargo.toml` and enabling only those features
/// that they wish to use.
///
/// [`DefaultPlugins`] contains all the plugins typically required to build
/// a *Bevy* application which includes a *window* and presentation components.
/// For *headless* cases – without a *window* or presentation, see [`MinimalPlugins`].
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self, mut set: PluginSet) -> PluginSet {
        set = set.add_plugins((
            bevy_log::LogPlugin::default(),
            bevy_core::TaskPoolPlugin::default(),
            bevy_core::TypeRegistrationPlugin,
            bevy_core::FrameCountPlugin,
            bevy_time::TimePlugin,
            bevy_transform::TransformPluginGroup,
            bevy_hierarchy::HierarchyPlugin,
            bevy_diagnostic::DiagnosticsPlugin,
            bevy_input::InputPlugin,
            bevy_window::WindowPlugin::default(),
            bevy_a11y::AccessibilityPlugin,
            IgnoreAmbiguitiesPlugin,
        ));

        #[cfg(feature = "bevy_asset")]
        {
            set = set.add_plugins(bevy_asset::AssetPlugin::default());
        }

        #[cfg(feature = "bevy_scene")]
        {
            set = set.add_plugins(bevy_scene::ScenePlugin);
        }

        #[cfg(feature = "bevy_winit")]
        {
            set = set.add_plugins(bevy_winit::WinitPluginGroup::default());
        }

        #[cfg(feature = "bevy_render")]
        {
            set = set.add_plugins((
                bevy_render::RenderPlugin::default(),
                bevy_render::texture::ImagePlugin::default(),
            ));

            #[cfg(all(not(target_arch = "wasm32"), feature = "multi-threaded"))]
            {
                set = set.add_plugins(bevy_render::pipelined_rendering::PipelinedRenderingPlugin);
            }
        }

        #[cfg(feature = "bevy_core_pipeline")]
        {
            set = set.add_plugins(bevy_core_pipeline::CorePipelinePlugin);
        }

        #[cfg(feature = "bevy_sprite")]
        {
            set = set.add_plugins(bevy_sprite::SpritePlugin);
        }

        #[cfg(feature = "bevy_text")]
        {
            set = set.add_plugins(bevy_text::TextPlugin);
        }

        #[cfg(feature = "bevy_ui")]
        {
            set = set.add_plugins(bevy_ui::UiPlugin);
        }

        #[cfg(feature = "bevy_pbr")]
        {
            set = set.add_plugins(bevy_pbr::PbrPlugin::default());
        }

        // NOTE: Load this after renderer initialization so that it knows about the supported
        // compressed texture formats
        #[cfg(feature = "bevy_gltf")]
        {
            set = set.add_plugins(bevy_gltf::GltfPlugin::default());
        }

        #[cfg(feature = "bevy_audio")]
        {
            set = set.add_plugins(bevy_audio::AudioPlugin::default());
        }

        #[cfg(feature = "bevy_gilrs")]
        {
            set = set.add_plugins(bevy_gilrs::GilrsPlugin);
        }

        #[cfg(feature = "bevy_animation")]
        {
            set = set.add_plugins(bevy_animation::AnimationPlugin);
        }

        #[cfg(feature = "bevy_gizmos")]
        {
            set = set.add_plugins(bevy_gizmos::GizmoPlugin);
        }

        set
    }
}

struct IgnoreAmbiguitiesPlugin;

impl Plugin for IgnoreAmbiguitiesPlugin {
    #[allow(unused_variables)] // Variables are used depending on enabled features
    fn build(&self, app: &mut bevy_app::App) {
        // bevy_ui owns the Transform and cannot be animated
        #[cfg(all(feature = "bevy_animation", feature = "bevy_ui"))]
        app.ignore_ambiguity(
            bevy_app::PostUpdate,
            bevy_animation::animation_player,
            bevy_ui::ui_layout_system,
        );

        #[cfg(feature = "bevy_render")]
        if let Ok(render_app) = app.get_sub_app_mut(bevy_render::RenderApp) {
            #[cfg(all(feature = "bevy_gizmos", feature = "bevy_sprite"))]
            {
                render_app.ignore_ambiguity(
                    bevy_render::Render,
                    bevy_gizmos::GizmoRenderSystem::QueueLineGizmos2d,
                    bevy_sprite::queue_sprites,
                );
                render_app.ignore_ambiguity(
                    bevy_render::Render,
                    bevy_gizmos::GizmoRenderSystem::QueueLineGizmos2d,
                    bevy_sprite::queue_material2d_meshes::<bevy_sprite::ColorMaterial>,
                );
            }
            #[cfg(all(feature = "bevy_gizmos", feature = "bevy_pbr"))]
            {
                render_app.ignore_ambiguity(
                    bevy_render::Render,
                    bevy_gizmos::GizmoRenderSystem::QueueLineGizmos3d,
                    bevy_pbr::queue_material_meshes::<bevy_pbr::StandardMaterial>,
                );
            }
        }
    }
}

/// This plugin group will add the minimal plugins for a *Bevy* application:
/// * [`TaskPoolPlugin`](crate::core::TaskPoolPlugin)
/// * [`TypeRegistrationPlugin`](crate::core::TypeRegistrationPlugin)
/// * [`FrameCountPlugin`](crate::core::FrameCountPlugin)
/// * [`TimePlugin`](crate::time::TimePlugin)
/// * [`ScheduleRunnerPlugin`](crate::app::ScheduleRunnerPlugin)
///
/// This group of plugins is intended for use for minimal, *headless* programs –
/// see the [*Bevy* *headless* example](https://github.com/bevyengine/bevy/blob/main/examples/app/headless.rs)
/// – and includes a [schedule runner (`ScheduleRunnerPlugin`)](crate::app::ScheduleRunnerPlugin)
/// to provide functionality that would otherwise be driven by a windowed application's
/// *event loop* or *message loop*.
///
/// Windowed applications that wish to use a reduced set of plugins should consider the
/// [`DefaultPlugins`] plugin group which can be controlled with *Cargo* *feature* flags.
pub struct MinimalPlugins;

impl PluginGroup for MinimalPlugins {
    fn build(self, set: PluginSet) -> PluginSet {
        set.add_plugins((
            bevy_core::TaskPoolPlugin::default(),
            bevy_core::TypeRegistrationPlugin,
            bevy_core::FrameCountPlugin,
            bevy_time::TimePlugin,
            bevy_app::ScheduleRunnerPlugin::default(),
        ))
    }
}
