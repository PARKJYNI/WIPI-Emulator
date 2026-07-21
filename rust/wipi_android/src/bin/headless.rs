//! Milestone 2: run a game headlessly on an Android device (via adb shell)
//! and dump the first painted frame to a BMP file.
//!
//! Usage: headless <game file> <output bmp> [ticks]

use std::{fs, io::Write, path::PathBuf, sync::Arc};

use wie_backend::Event;

use wipi_android::platform::{CapturedFrame, MobilePlatform, SharedPlatform};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();
    let [_, game_path, out_path, rest @ ..] = args.as_slice() else {
        anyhow::bail!("usage: headless <game file> <output bmp prefix> [seconds]");
    };
    let max_seconds: u64 = rest.first().map(|x| x.parse()).transpose()?.unwrap_or(10);

    let out_path_buf = PathBuf::from(out_path);
    let data_dir = out_path_buf.parent().unwrap_or(std::path::Path::new(".")).join("wie_data");
    let platform = Arc::new(MobilePlatform::new(data_dir, 240, 320, None));

    let buf = fs::read(game_path)?;
    let mut emulator = wipi_android::create_emulator(Box::new(SharedPlatform(platform.clone())), game_path, &buf)?;

    let start = std::time::Instant::now();
    let mut last_frame: Option<CapturedFrame> = None;
    let mut saved_seconds = 0u64;
    let mut ticks = 0u64;
    while start.elapsed().as_secs() < max_seconds {
        if platform.screen_capture().take_redraw_request() {
            emulator.handle_event(Event::Redraw);
        }

        emulator.tick()?;
        ticks += 1;

        if let Some(frame) = platform.screen_capture().take_frame() {
            last_frame = Some(frame);
        }

        // 1초마다 현재 프레임 저장 → 진행 과정 관찰용
        let elapsed = start.elapsed().as_secs();
        if elapsed > saved_seconds {
            saved_seconds = elapsed;
            if let Some(frame) = &last_frame {
                let non_black = frame.pixels.chunks(4).filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0).count();
                write_bmp(&format!("{out_path}_{saved_seconds:03}.bmp"), frame)?;
                eprintln!("t={saved_seconds}s ticks={ticks} non_black_pixels={non_black}");
            } else {
                eprintln!("t={saved_seconds}s ticks={ticks} (no frame painted yet)");
            }
        }
    }

    let Some(frame) = last_frame else {
        anyhow::bail!("no frame was painted after {max_seconds}s ({ticks} ticks)");
    };

    write_bmp(&format!("{out_path}_final.bmp"), &frame)?;
    println!(
        "OK: {}x{} final frame written to {out_path}_final.bmp after {ticks} ticks",
        frame.width, frame.height
    );

    Ok(())
}

/// 24-bit BMP writer (bottom-up, BGR) — 의존성 없이 프레임 덤프용
fn write_bmp(path: &str, frame: &CapturedFrame) -> anyhow::Result<()> {
    let (w, h) = (frame.width as usize, frame.height as usize);
    let row_size = (w * 3).div_ceil(4) * 4;
    let pixel_data_size = row_size * h;
    let file_size = 54 + pixel_data_size;

    let mut out = Vec::with_capacity(file_size);
    // BITMAPFILEHEADER
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(file_size as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    // BITMAPINFOHEADER
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0; 24]);

    for y in (0..h).rev() {
        let row_start = y * w * 4;
        for x in 0..w {
            let i = row_start + x * 4;
            out.extend_from_slice(&[frame.pixels[i + 2], frame.pixels[i + 1], frame.pixels[i]]);
        }
        out.resize(out.len() + (row_size - w * 3), 0);
    }

    let mut file = fs::File::create(path)?;
    file.write_all(&out)?;
    Ok(())
}
