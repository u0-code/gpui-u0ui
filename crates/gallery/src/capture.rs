//! `--capture <dir>`: render gallery routes to PNG files without presenting
//! them, through GPUI's `Window::render_to_image` (test-support) on the
//! native renderer. The window is created hidden; each route is drawn by
//! hand (a hidden window gets no frame callbacks), given time for its
//! animations to settle, drawn again and saved as `<dir>/<id>-<theme>.png`.
use crate::shell::{Gallery, Route};
use anyhow::Context as _;
use gpui_kit::{App, Entity, Window};
use std::path::PathBuf;
use std::time::Duration;

/// What to capture.
#[derive(Clone, Debug)]
pub struct Capture {
    /// The output directory.
    pub dir: PathBuf,
    /// The routes, in order.
    pub routes: Vec<Route>,
    /// The file name suffix (the theme id).
    pub suffix: String,
}

/// Draw the window once by hand.
fn draw(window: &mut Window, cx: &mut App) {
    window.refresh();
    window.draw(cx).clear(cx);
}

/// Capture every route of `capture` from `gallery`'s window, then quit.
pub fn start(capture: Capture, gallery: Entity<Gallery>, window: &mut Window, cx: &mut App) {
    window
        .spawn(cx, async move |cx| {
            let result: anyhow::Result<()> = async {
                std::fs::create_dir_all(&capture.dir)
                    .with_context(|| format!("creating {}", capture.dir.display()))?;
                for route in &capture.routes {
                    let route = *route;
                    cx.update(|_, cx| gallery.update(cx, |g, cx| g.navigate(route, cx)))?;
                    // Lay out, let tabs measure themselves and animations settle.
                    for _ in 0..4 {
                        cx.background_executor()
                            .timer(Duration::from_millis(16))
                            .await;
                        cx.update(draw)?;
                    }
                    cx.background_executor()
                        .timer(Duration::from_millis(600))
                        .await;
                    let path = capture
                        .dir
                        .join(format!("{}-{}.png", route.id(), capture.suffix));
                    cx.update(|window, cx| -> anyhow::Result<()> {
                        draw(window, cx);
                        draw(window, cx);
                        let image = window
                            .render_to_image()
                            .context("native GPU image capture failed")?;
                        image
                            .save(&path)
                            .with_context(|| format!("saving {}", path.display()))?;
                        Ok(())
                    })??;
                    println!("{}", path.display());
                }
                Ok(())
            }
            .await;
            if let Err(error) = result {
                eprintln!("capture failed: {error:#}");
                std::process::exit(1);
            }
            let _ = cx.update(|_, cx| cx.quit());
        })
        .detach();
}
