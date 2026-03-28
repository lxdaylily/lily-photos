use image::{
    ColorType, DynamicImage, GenericImageView, ImageEncoder, Rgb, RgbImage, imageops::FilterType,
};

const MAX_EDGE: u32 = 2560;
const JPEG_QUALITY: u8 = 82;

pub struct ProcessedImage {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub filename: Option<String>,
}

pub async fn optimize_image_for_storage(
    filename: Option<String>,
    mime_type: String,
    data: Vec<u8>,
) -> Result<ProcessedImage, String> {
    tokio::task::spawn_blocking(move || {
        let image = image::load_from_memory(&data).map_err(|err| err.to_string())?;
        let optimized = resize_if_needed(image);
        let rgb = flatten_to_rgb(optimized);

        let mut encoded = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded, JPEG_QUALITY)
            .write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                ColorType::Rgb8.into(),
            )
            .map_err(|err| err.to_string())?;

        let use_encoded = encoded.len() < data.len();
        let final_data = if use_encoded { encoded } else { data };
        let final_mime = if use_encoded {
            "image/jpeg".to_string()
        } else {
            mime_type
        };
        let final_filename = if use_encoded {
            filename.map(rewrite_filename_to_jpeg)
        } else {
            filename
        };

        Ok(ProcessedImage {
            data: final_data,
            mime_type: final_mime,
            filename: final_filename,
        })
    })
    .await
    .map_err(|err| err.to_string())?
}

fn resize_if_needed(image: DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();
    let longest = width.max(height);

    if longest <= MAX_EDGE {
        return image;
    }

    let scale = MAX_EDGE as f32 / longest as f32;
    let target_width = ((width as f32) * scale).round() as u32;
    let target_height = ((height as f32) * scale).round() as u32;

    image.resize(target_width.max(1), target_height.max(1), FilterType::Lanczos3)
}

fn flatten_to_rgb(image: DynamicImage) -> RgbImage {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut rgb = RgbImage::new(width, height);

    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        let alpha = a as f32 / 255.0;
        let blend = |channel: u8| -> u8 {
            ((channel as f32 * alpha) + (255.0 * (1.0 - alpha))).round() as u8
        };

        rgb.put_pixel(x, y, Rgb([blend(r), blend(g), blend(b)]));
    }

    rgb
}

fn rewrite_filename_to_jpeg(filename: String) -> String {
    let trimmed = filename.trim();
    match trimmed.rsplit_once('.') {
        Some((base, _)) if !base.is_empty() => format!("{base}.jpg"),
        _ => format!("{trimmed}.jpg"),
    }
}
