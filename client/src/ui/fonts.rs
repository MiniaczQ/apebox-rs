use bevy::asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy_egui::EguiContext;
use common::game::CustomFont;
use egui::{FontData, FontFamily, FontId};
use thiserror::Error;

use crate::{
    barrier::{AppExtBarrier, Barrier},
    states::InitialResources,
};

pub struct FontsPlugin;

impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<EguiFont>();
        app.init_asset_loader::<EguiFontLoader>();
        app.add_barrier_system::<InitialResources, _>(load_fonts);
    }
}

pub const FONTS: [&str; 6] = [
    "fonts/BLADRMF_.TTF",
    "fonts/IHATCS__.TTF",
    "fonts/Lemon Shake Shake.ttf",
    "fonts/LittleKidsHandwriting-Regular.otf",
    "fonts/Next Bravo.ttf",
    "fonts/whitrabt.ttf",
];

pub trait IntoFontFamily {
    fn get_family(&self) -> FontFamily;
    fn get_font_id(&self) -> FontId;
}

impl IntoFontFamily for CustomFont {
    fn get_family(&self) -> FontFamily {
        let font = FONTS[self.0];
        FontFamily::Name(font.into())
    }

    fn get_font_id(&self) -> FontId {
        FontId::new(24.0, self.get_family())
    }
}

fn load_fonts(
    mut egui_fonts: ResMut<Assets<EguiFont>>,
    mut local_handles: Local<Option<Vec<Handle<EguiFont>>>>,
    mut ui_ctx: Query<&mut EguiContext>,
    mut barrier: Barrier<InitialResources>,
    asset_server: Res<AssetServer>,
) {
    if barrier.is_completed() {
        return;
    }

    // Queue asset loading
    let Some(handles) = local_handles.as_mut() else {
        let mut handles: Vec<Handle<EguiFont>> = vec![];
        for path in FONTS {
            handles.push(asset_server.load(path));
        }
        *local_handles = Some(handles);
        return;
    };

    // Await until assets are loaded
    let all_loaded = handles.iter().all(|h| egui_fonts.contains(h));
    if !all_loaded {
        return;
    }

    // Final processing
    let mut fonts = egui::FontDefinitions::default();
    let handles = local_handles.take().unwrap();
    for (handle, name) in handles.into_iter().zip(FONTS) {
        let font_data = egui_fonts.remove(&handle).unwrap().0;
        fonts.font_data.insert(name.into(), font_data);
        let family = FontFamily::Name(name.into());
        fonts.families.entry(family).or_default().push(name.into());
    }
    ui_ctx.single_mut().get_mut().set_fonts(fonts);

    barrier.complete();
}

#[derive(Asset, TypePath)]
pub struct EguiFont(FontData);

#[derive(Default)]
pub struct EguiFontLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum EguiFontLoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl AssetLoader for EguiFontLoader {
    type Asset = EguiFont;
    type Settings = ();
    type Error = EguiFontLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<EguiFont, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(EguiFont(FontData::from_owned(bytes)))
    }

    fn extensions(&self) -> &[&str] {
        &["ttf", "otf"]
    }
}
