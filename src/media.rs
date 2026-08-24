//! High-level media-message builders.
//!
//! Turn an [`UploadResponse`] (from [`Client::upload`](crate::Client::upload)) plus
//! typed options into a ready-to-send [`wa::Message`], so callers don't hand-assemble
//! the CDN/crypto fields (url, direct_path, media_key, file_sha256, file_enc_sha256,
//! file_length, media_key_timestamp, streaming_sidecar) every time. Mirrors WA Web's
//! send-media path, which builds the proto from the upload result internally.
//! The resulting [`wa::Message`] is sent with `client.send_message(to, msg)`.
//!
//! ```no_run
//! # fn build(upload: whatsapp_rust::upload::UploadResponse) {
//! use whatsapp_rust::media::{self, ImageOptions};
//! let _msg = media::image_message(upload, ImageOptions { caption: Some("hi".into()), ..Default::default() });
//! # }
//! ```

use crate::upload::UploadResponse;
use waproto::whatsapp as wa;

#[derive(Debug, Clone, Default)]
pub struct ImageOptions {
    pub caption: Option<String>,
    /// Defaults to `image/jpeg`.
    pub mimetype: Option<String>,
    pub jpeg_thumbnail: Option<Vec<u8>>,
    pub context_info: Option<Box<wa::ContextInfo>>,
}

#[derive(Debug, Clone, Default)]
pub struct VideoOptions {
    pub caption: Option<String>,
    /// Defaults to `video/mp4`.
    pub mimetype: Option<String>,
    pub jpeg_thumbnail: Option<Vec<u8>>,
    pub duration_seconds: Option<u32>,
    /// Send as a looping GIF-style clip.
    pub gif_playback: Option<bool>,
    pub context_info: Option<Box<wa::ContextInfo>>,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentOptions {
    /// Defaults to `application/octet-stream`.
    pub mimetype: Option<String>,
    /// File name shown to the recipient.
    pub file_name: Option<String>,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub page_count: Option<u32>,
    pub jpeg_thumbnail: Option<Vec<u8>>,
    pub context_info: Option<Box<wa::ContextInfo>>,
}

#[derive(Debug, Clone, Default)]
pub struct AudioOptions {
    /// Defaults to `audio/ogg; codecs=opus`.
    pub mimetype: Option<String>,
    pub duration_seconds: Option<u32>,
    /// Push-to-talk (voice note) flag.
    pub ptt: Option<bool>,
    /// PCM waveform preview bytes (voice notes).
    pub waveform: Option<Vec<u8>>,
    pub context_info: Option<Box<wa::ContextInfo>>,
}

#[derive(Debug, Clone, Default)]
pub struct StickerOptions {
    /// Defaults to `image/webp`.
    pub mimetype: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Animated WebP flag — without it clients render only the first frame.
    pub is_animated: Option<bool>,
    pub is_lottie: Option<bool>,
    /// PNG preview used in reply quotes (stickers use PNG, not JPEG).
    pub png_thumbnail: Option<Vec<u8>>,
    pub accessibility_label: Option<String>,
    pub context_info: Option<Box<wa::ContextInfo>>,
}

impl StickerOptions {
    /// Prefill `width`/`height`/`is_animated`/`mimetype` by sniffing the WebP
    /// header, so bubbles don't collapse from missing dimensions.
    pub fn from_webp(data: &[u8]) -> Self {
        let dims = wacore::webp::dimensions(data);
        Self {
            mimetype: Some("image/webp".to_string()),
            width: dims.map(|(w, _)| w),
            height: dims.map(|(_, h)| h),
            is_animated: Some(wacore::webp::is_animated(data)),
            ..Default::default()
        }
    }
}

/// CDN/crypto descriptor of an already-uploaded sticker, for re-sending
/// without a fresh upload — the same thing WA Web does for favorited and
/// recent stickers (a `StickerAction` carries exactly these fields).
/// [`UploadResponse`] is `#[non_exhaustive]`, so re-senders build this
/// instead.
#[derive(Debug, Clone)]
pub struct StickerRef {
    pub url: Option<String>,
    pub direct_path: String,
    pub media_key: Vec<u8>,
    pub file_sha256: Vec<u8>,
    pub file_enc_sha256: Vec<u8>,
    pub file_length: u64,
    pub media_key_timestamp: Option<i64>,
}

impl From<UploadResponse> for StickerRef {
    fn from(upload: UploadResponse) -> Self {
        Self {
            url: Some(upload.url),
            direct_path: upload.direct_path,
            media_key: upload.media_key.to_vec(),
            file_sha256: upload.file_sha256.to_vec(),
            file_enc_sha256: upload.file_enc_sha256.to_vec(),
            file_length: upload.file_length,
            media_key_timestamp: Some(upload.media_key_timestamp),
        }
    }
}

/// Build a sticker message from an upload result.
pub fn sticker_message(upload: UploadResponse, opts: StickerOptions) -> wa::Message {
    sticker_message_from_ref(upload.into(), opts)
}

/// Build a sticker message from an existing CDN descriptor (no upload).
/// `sticker_sent_ts` is stamped here, at build time, the way WA Web does —
/// recipients use it to order their recent-stickers tray.
pub fn sticker_message_from_ref(sticker: StickerRef, opts: StickerOptions) -> wa::Message {
    wa::Message {
        sticker_message: buffa::MessageField::some(wa::message::StickerMessage {
            url: sticker.url,
            direct_path: Some(sticker.direct_path),
            media_key: Some(sticker.media_key),
            file_sha256: Some(sticker.file_sha256),
            file_enc_sha256: Some(sticker.file_enc_sha256),
            file_length: Some(sticker.file_length),
            media_key_timestamp: sticker.media_key_timestamp,
            mimetype: Some(opts.mimetype.unwrap_or_else(|| "image/webp".to_string())),
            width: opts.width,
            height: opts.height,
            is_animated: opts.is_animated,
            is_lottie: opts.is_lottie,
            png_thumbnail: opts.png_thumbnail,
            accessibility_label: opts.accessibility_label,
            sticker_sent_ts: Some(wacore::time::now_millis()),
            context_info: opts
                .context_info
                .map(|ci| buffa::MessageField::some(*ci))
                .unwrap_or_default(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Build an image message from an upload result.
pub fn image_message(upload: UploadResponse, opts: ImageOptions) -> wa::Message {
    wa::Message {
        image_message: buffa::MessageField::some(wa::message::ImageMessage {
            url: Some(upload.url),
            direct_path: Some(upload.direct_path),
            media_key: Some(upload.media_key.to_vec()),
            file_sha256: Some(upload.file_sha256.to_vec()),
            file_enc_sha256: Some(upload.file_enc_sha256.to_vec()),
            file_length: Some(upload.file_length),
            media_key_timestamp: Some(upload.media_key_timestamp),
            mimetype: Some(opts.mimetype.unwrap_or_else(|| "image/jpeg".to_string())),
            caption: opts.caption,
            jpeg_thumbnail: opts.jpeg_thumbnail,
            context_info: opts
                .context_info
                .map(|ci| buffa::MessageField::some(*ci))
                .unwrap_or_default(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Build a video message from an upload result. Carries the streaming sidecar
/// (progressive-playback HMAC table) from the upload when present.
pub fn video_message(upload: UploadResponse, opts: VideoOptions) -> wa::Message {
    wa::Message {
        video_message: buffa::MessageField::some(wa::message::VideoMessage {
            url: Some(upload.url),
            direct_path: Some(upload.direct_path),
            media_key: Some(upload.media_key.to_vec()),
            file_sha256: Some(upload.file_sha256.to_vec()),
            file_enc_sha256: Some(upload.file_enc_sha256.to_vec()),
            file_length: Some(upload.file_length),
            media_key_timestamp: Some(upload.media_key_timestamp),
            streaming_sidecar: upload.streaming_sidecar,
            mimetype: Some(opts.mimetype.unwrap_or_else(|| "video/mp4".to_string())),
            caption: opts.caption,
            jpeg_thumbnail: opts.jpeg_thumbnail,
            seconds: opts.duration_seconds,
            gif_playback: opts.gif_playback,
            context_info: opts
                .context_info
                .map(|ci| buffa::MessageField::some(*ci))
                .unwrap_or_default(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Build a document message from an upload result.
pub fn document_message(upload: UploadResponse, opts: DocumentOptions) -> wa::Message {
    wa::Message {
        document_message: buffa::MessageField::some(wa::message::DocumentMessage {
            url: Some(upload.url),
            direct_path: Some(upload.direct_path),
            media_key: Some(upload.media_key.to_vec()),
            file_sha256: Some(upload.file_sha256.to_vec()),
            file_enc_sha256: Some(upload.file_enc_sha256.to_vec()),
            file_length: Some(upload.file_length),
            media_key_timestamp: Some(upload.media_key_timestamp),
            mimetype: Some(
                opts.mimetype
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
            ),
            file_name: opts.file_name,
            title: opts.title,
            caption: opts.caption,
            page_count: opts.page_count,
            jpeg_thumbnail: opts.jpeg_thumbnail,
            context_info: opts
                .context_info
                .map(|ci| buffa::MessageField::some(*ci))
                .unwrap_or_default(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Build an audio / voice-note message from an upload result. Carries the
/// streaming sidecar from the upload when present.
pub fn audio_message(upload: UploadResponse, opts: AudioOptions) -> wa::Message {
    wa::Message {
        audio_message: buffa::MessageField::some(wa::message::AudioMessage {
            url: Some(upload.url),
            direct_path: Some(upload.direct_path),
            media_key: Some(upload.media_key.to_vec()),
            file_sha256: Some(upload.file_sha256.to_vec()),
            file_enc_sha256: Some(upload.file_enc_sha256.to_vec()),
            file_length: Some(upload.file_length),
            media_key_timestamp: Some(upload.media_key_timestamp),
            streaming_sidecar: upload.streaming_sidecar,
            mimetype: Some(
                opts.mimetype
                    .unwrap_or_else(|| "audio/ogg; codecs=opus".to_string()),
            ),
            seconds: opts.duration_seconds,
            ptt: opts.ptt,
            waveform: opts.waveform,
            context_info: opts
                .context_info
                .map(|ci| buffa::MessageField::some(*ci))
                .unwrap_or_default(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_upload() -> UploadResponse {
        UploadResponse {
            url: "https://cdn/u".into(),
            direct_path: "/d".into(),
            media_key: [1u8; 32],
            file_enc_sha256: [2u8; 32],
            file_sha256: [3u8; 32],
            file_length: 4096,
            media_key_timestamp: 1_700_000_000,
            streaming_sidecar: Some(vec![9, 9, 9]),
        }
    }

    #[test]
    fn image_maps_cdn_fields_and_defaults_mimetype() {
        let msg = image_message(sample_upload(), ImageOptions::default());
        let im = msg.image_message.unwrap();
        assert_eq!(im.url.as_deref(), Some("https://cdn/u"));
        assert_eq!(im.direct_path.as_deref(), Some("/d"));
        assert_eq!(im.media_key.as_deref(), Some(&[1u8; 32][..]));
        assert_eq!(im.file_sha256.as_deref(), Some(&[3u8; 32][..]));
        assert_eq!(im.file_enc_sha256.as_deref(), Some(&[2u8; 32][..]));
        assert_eq!(im.file_length, Some(4096));
        assert_eq!(im.media_key_timestamp, Some(1_700_000_000));
        assert_eq!(im.mimetype.as_deref(), Some("image/jpeg"));
    }

    #[test]
    fn video_carries_sidecar_and_options() {
        let msg = video_message(
            sample_upload(),
            VideoOptions {
                caption: Some("c".into()),
                duration_seconds: Some(12),
                gif_playback: Some(true),
                ..Default::default()
            },
        );
        let vm = msg.video_message.unwrap();
        assert_eq!(vm.streaming_sidecar.as_deref(), Some(&[9, 9, 9][..]));
        assert_eq!(vm.seconds, Some(12));
        assert_eq!(vm.gif_playback, Some(true));
        assert_eq!(vm.caption.as_deref(), Some("c"));
        assert_eq!(vm.mimetype.as_deref(), Some("video/mp4"));
    }

    #[test]
    fn document_and_audio_set_type_specific_fields() {
        let doc = document_message(
            sample_upload(),
            DocumentOptions {
                file_name: Some("f.pdf".into()),
                page_count: Some(3),
                ..Default::default()
            },
        )
        .document_message
        .unwrap();
        assert_eq!(doc.file_name.as_deref(), Some("f.pdf"));
        assert_eq!(doc.page_count, Some(3));
        assert_eq!(doc.mimetype.as_deref(), Some("application/octet-stream"));

        let audio = audio_message(
            sample_upload(),
            AudioOptions {
                ptt: Some(true),
                duration_seconds: Some(5),
                ..Default::default()
            },
        )
        .audio_message
        .unwrap();
        assert_eq!(audio.ptt, Some(true));
        assert_eq!(audio.seconds, Some(5));
        assert_eq!(audio.streaming_sidecar.as_deref(), Some(&[9, 9, 9][..]));
    }

    #[test]
    fn image_maps_context_info() {
        let context = Box::new(wa::ContextInfo::default());

        let image_msg = image_message(
            sample_upload(),
            ImageOptions {
                context_info: Some(context),
                ..Default::default()
            },
        );

        let image = image_msg.image_message.unwrap();

        assert!(image.context_info.is_set());
    }

    #[test]
    fn video_maps_context_info() {
        let context = Box::new(wa::ContextInfo::default());

        let video_msg = video_message(
            sample_upload(),
            VideoOptions {
                context_info: Some(context),
                ..Default::default()
            },
        );

        let video = video_msg.video_message.unwrap();

        assert!(video.context_info.is_set());
    }

    fn webp_512(animated: bool) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"WEBP");
        buf.extend_from_slice(b"VP8X");
        buf.extend_from_slice(&10u32.to_le_bytes());
        buf.push(if animated { 0x02 } else { 0x00 });
        buf.extend_from_slice(&[0u8; 3]);
        buf.extend_from_slice(&511u32.to_le_bytes()[..3]);
        buf.extend_from_slice(&511u32.to_le_bytes()[..3]);
        let riff_size = (buf.len() - 8) as u32;
        buf[4..8].copy_from_slice(&riff_size.to_le_bytes());
        buf
    }

    #[test]
    fn sticker_maps_cdn_fields_and_defaults_mimetype() {
        let msg = sticker_message(sample_upload(), StickerOptions::default());
        let sm = msg.sticker_message.unwrap();
        assert_eq!(sm.url.as_deref(), Some("https://cdn/u"));
        assert_eq!(sm.direct_path.as_deref(), Some("/d"));
        assert_eq!(sm.media_key.as_deref(), Some(&[1u8; 32][..]));
        assert_eq!(sm.file_sha256.as_deref(), Some(&[3u8; 32][..]));
        assert_eq!(sm.file_enc_sha256.as_deref(), Some(&[2u8; 32][..]));
        assert_eq!(sm.file_length, Some(4096));
        assert_eq!(sm.media_key_timestamp, Some(1_700_000_000));
        assert_eq!(sm.mimetype.as_deref(), Some("image/webp"));
        assert!(sm.sticker_sent_ts.is_some());
    }

    #[test]
    fn sticker_options_from_webp_sniffs_metadata() {
        let opts = StickerOptions::from_webp(&webp_512(true));
        assert_eq!(opts.width, Some(512));
        assert_eq!(opts.height, Some(512));
        assert_eq!(opts.is_animated, Some(true));
        assert_eq!(opts.mimetype.as_deref(), Some("image/webp"));

        let opts = StickerOptions::from_webp(&webp_512(false));
        assert_eq!(opts.is_animated, Some(false));

        let opts = StickerOptions::from_webp(b"not webp");
        assert_eq!(opts.width, None);
        assert_eq!(opts.height, None);
    }

    #[test]
    fn sticker_from_ref_without_url_or_key_timestamp() {
        let msg = sticker_message_from_ref(
            StickerRef {
                url: None,
                direct_path: "/d2".into(),
                media_key: vec![7u8; 32],
                file_sha256: vec![8u8; 32],
                file_enc_sha256: vec![9u8; 32],
                file_length: 123,
                media_key_timestamp: None,
            },
            StickerOptions {
                width: Some(512),
                height: Some(512),
                is_animated: Some(false),
                ..Default::default()
            },
        );
        let sm = msg.sticker_message.unwrap();
        assert_eq!(sm.url, None);
        assert_eq!(sm.media_key_timestamp, None);
        assert_eq!(sm.direct_path.as_deref(), Some("/d2"));
        assert_eq!(sm.width, Some(512));
        assert!(sm.sticker_sent_ts.is_some());
    }

    #[test]
    fn sticker_maps_context_info_and_classifies_as_sticker() {
        let msg = sticker_message(
            sample_upload(),
            StickerOptions {
                context_info: Some(Box::new(wa::ContextInfo::default())),
                ..Default::default()
            },
        );
        assert_eq!(wacore::send::media_type_from_message(&msg), Some("sticker"));
        assert!(msg.sticker_message.unwrap().context_info.is_set());
    }
}
