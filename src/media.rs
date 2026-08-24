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
//! let _msg = media::image_message(upload, ImageOptions::default().with_caption("hi"));
//! # }
//! ```

use crate::upload::UploadResponse;
use waproto::whatsapp as wa;

/// Optional image metadata. An empty configuration uses the default MIME type.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ImageOptions {
    pub caption: Option<String>,
    /// Defaults to `image/jpeg`.
    pub mimetype: Option<String>,
    pub jpeg_thumbnail: Option<Vec<u8>>,
    pub context_info: Option<Box<wa::ContextInfo>>,
}

/// Optional video metadata. An empty configuration uses the default MIME type.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
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

/// Optional document metadata. An empty configuration uses the default MIME type.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
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

/// Optional audio metadata. An empty configuration uses the default MIME type.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
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

impl ImageOptions {
    /// Set the image caption.
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Override the default MIME type.
    pub fn with_mimetype(mut self, mimetype: impl Into<String>) -> Self {
        self.mimetype = Some(mimetype.into());
        self
    }

    /// Set a JPEG thumbnail.
    pub fn with_jpeg_thumbnail(mut self, thumbnail: Vec<u8>) -> Self {
        self.jpeg_thumbnail = Some(thumbnail);
        self
    }

    /// Attach reply, mention or other message context.
    pub fn with_context_info(mut self, context_info: Box<wa::ContextInfo>) -> Self {
        self.context_info = Some(context_info);
        self
    }
}

impl VideoOptions {
    /// Set the video caption.
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Override the default MIME type.
    pub fn with_mimetype(mut self, mimetype: impl Into<String>) -> Self {
        self.mimetype = Some(mimetype.into());
        self
    }

    /// Set a JPEG thumbnail.
    pub fn with_jpeg_thumbnail(mut self, thumbnail: Vec<u8>) -> Self {
        self.jpeg_thumbnail = Some(thumbnail);
        self
    }

    /// Set the clip duration in seconds.
    pub fn with_duration_seconds(mut self, seconds: u32) -> Self {
        self.duration_seconds = Some(seconds);
        self
    }

    /// Set whether to send as a looping GIF-style clip.
    pub fn with_gif_playback(mut self, enabled: bool) -> Self {
        self.gif_playback = Some(enabled);
        self
    }

    /// Attach reply, mention or other message context.
    pub fn with_context_info(mut self, context_info: Box<wa::ContextInfo>) -> Self {
        self.context_info = Some(context_info);
        self
    }
}

impl DocumentOptions {
    /// Override the default MIME type.
    pub fn with_mimetype(mut self, mimetype: impl Into<String>) -> Self {
        self.mimetype = Some(mimetype.into());
        self
    }

    /// Set the file name shown to the recipient.
    pub fn with_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
    }

    /// Set the document title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the document caption.
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Set the document page count.
    pub fn with_page_count(mut self, page_count: u32) -> Self {
        self.page_count = Some(page_count);
        self
    }

    /// Set a JPEG thumbnail.
    pub fn with_jpeg_thumbnail(mut self, thumbnail: Vec<u8>) -> Self {
        self.jpeg_thumbnail = Some(thumbnail);
        self
    }

    /// Attach reply, mention or other message context.
    pub fn with_context_info(mut self, context_info: Box<wa::ContextInfo>) -> Self {
        self.context_info = Some(context_info);
        self
    }
}

impl AudioOptions {
    /// Override the default MIME type.
    pub fn with_mimetype(mut self, mimetype: impl Into<String>) -> Self {
        self.mimetype = Some(mimetype.into());
        self
    }

    /// Set the audio duration in seconds.
    pub fn with_duration_seconds(mut self, seconds: u32) -> Self {
        self.duration_seconds = Some(seconds);
        self
    }

    /// Set the push-to-talk (voice note) flag.
    pub fn with_ptt(mut self, enabled: bool) -> Self {
        self.ptt = Some(enabled);
        self
    }

    /// Set waveform preview bytes.
    pub fn with_waveform(mut self, waveform: Vec<u8>) -> Self {
        self.waveform = Some(waveform);
        self
    }

    /// Attach reply, mention or other message context.
    pub fn with_context_info(mut self, context_info: Box<wa::ContextInfo>) -> Self {
        self.context_info = Some(context_info);
        self
    }
}

/// Optional sticker metadata. An empty configuration uses the default MIME type.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
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
        let mut opts = Self::default();
        opts.mimetype = Some("image/webp".to_string());
        opts.width = dims.map(|(w, _)| w);
        opts.height = dims.map(|(_, h)| h);
        opts.is_animated = Some(wacore::webp::is_animated(data));
        opts
    }

    /// Override the default MIME type.
    pub fn with_mimetype(mut self, mimetype: impl Into<String>) -> Self {
        self.mimetype = Some(mimetype.into());
        self
    }

    /// Set the sticker dimensions in pixels.
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set the animated WebP flag.
    pub fn with_is_animated(mut self, animated: bool) -> Self {
        self.is_animated = Some(animated);
        self
    }

    /// Set the Lottie flag.
    pub fn with_is_lottie(mut self, lottie: bool) -> Self {
        self.is_lottie = Some(lottie);
        self
    }

    /// Set the PNG preview used in reply quotes.
    pub fn with_png_thumbnail(mut self, thumbnail: Vec<u8>) -> Self {
        self.png_thumbnail = Some(thumbnail);
        self
    }

    /// Set the accessibility label.
    pub fn with_accessibility_label(mut self, label: impl Into<String>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Attach reply, mention or other message context.
    pub fn with_context_info(mut self, context_info: Box<wa::ContextInfo>) -> Self {
        self.context_info = Some(context_info);
        self
    }
}

/// CDN/crypto descriptor of an already-uploaded sticker, for re-sending
/// without a fresh upload — the same thing WA Web does for favorited and
/// recent stickers (a `StickerAction` carries exactly these fields).
/// [`UploadResponse`] is `#[non_exhaustive]`, so re-senders build this
/// instead; it stays a plain literal-constructible struct for that reason.
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
    let mut proto = wa::Message::default();
    proto.sticker_message = buffa::MessageField::some({
        let mut proto = wa::message::StickerMessage::default();
        proto.url = sticker.url;
        proto.direct_path = Some(sticker.direct_path);
        proto.media_key = Some(sticker.media_key);
        proto.file_sha256 = Some(sticker.file_sha256);
        proto.file_enc_sha256 = Some(sticker.file_enc_sha256);
        proto.file_length = Some(sticker.file_length);
        proto.media_key_timestamp = sticker.media_key_timestamp;
        proto.mimetype = Some(opts.mimetype.unwrap_or_else(|| "image/webp".to_string()));
        proto.width = opts.width;
        proto.height = opts.height;
        proto.is_animated = opts.is_animated;
        proto.is_lottie = opts.is_lottie;
        proto.png_thumbnail = opts.png_thumbnail;
        proto.accessibility_label = opts.accessibility_label;
        proto.sticker_sent_ts = Some(wacore::time::now_millis());
        proto.context_info = opts
            .context_info
            .map(|ci| buffa::MessageField::some(*ci))
            .unwrap_or_default();
        proto
    });
    proto
}

/// Build an image message from an upload result.
pub fn image_message(upload: UploadResponse, opts: ImageOptions) -> wa::Message {
    {
        let mut proto = wa::Message::default();
        proto.image_message = buffa::MessageField::some({
            let mut proto = wa::message::ImageMessage::default();
            proto.url = Some(upload.url);
            proto.direct_path = Some(upload.direct_path);
            proto.media_key = Some(upload.media_key.to_vec());
            proto.file_sha256 = Some(upload.file_sha256.to_vec());
            proto.file_enc_sha256 = Some(upload.file_enc_sha256.to_vec());
            proto.file_length = Some(upload.file_length);
            proto.media_key_timestamp = Some(upload.media_key_timestamp);
            proto.mimetype = Some(opts.mimetype.unwrap_or_else(|| "image/jpeg".to_string()));
            proto.caption = opts.caption;
            proto.jpeg_thumbnail = opts.jpeg_thumbnail;
            proto.context_info = opts
                .context_info
                .map(|ci| buffa::MessageField::some(*ci))
                .unwrap_or_default();
            proto
        });
        proto
    }
}

/// Build a video message from an upload result. Carries the streaming sidecar
/// (progressive-playback HMAC table) from the upload when present.
pub fn video_message(upload: UploadResponse, opts: VideoOptions) -> wa::Message {
    {
        let mut proto = wa::Message::default();
        proto.video_message = buffa::MessageField::some({
            let mut proto = wa::message::VideoMessage::default();
            proto.url = Some(upload.url);
            proto.direct_path = Some(upload.direct_path);
            proto.media_key = Some(upload.media_key.to_vec());
            proto.file_sha256 = Some(upload.file_sha256.to_vec());
            proto.file_enc_sha256 = Some(upload.file_enc_sha256.to_vec());
            proto.file_length = Some(upload.file_length);
            proto.media_key_timestamp = Some(upload.media_key_timestamp);
            proto.streaming_sidecar = upload.streaming_sidecar;
            proto.mimetype = Some(opts.mimetype.unwrap_or_else(|| "video/mp4".to_string()));
            proto.caption = opts.caption;
            proto.jpeg_thumbnail = opts.jpeg_thumbnail;
            proto.seconds = opts.duration_seconds;
            proto.gif_playback = opts.gif_playback;
            proto.context_info = opts
                .context_info
                .map(|ci| buffa::MessageField::some(*ci))
                .unwrap_or_default();
            proto
        });
        proto
    }
}

/// Build a document message from an upload result.
pub fn document_message(upload: UploadResponse, opts: DocumentOptions) -> wa::Message {
    {
        let mut proto = wa::Message::default();
        proto.document_message = buffa::MessageField::some({
            let mut proto = wa::message::DocumentMessage::default();
            proto.url = Some(upload.url);
            proto.direct_path = Some(upload.direct_path);
            proto.media_key = Some(upload.media_key.to_vec());
            proto.file_sha256 = Some(upload.file_sha256.to_vec());
            proto.file_enc_sha256 = Some(upload.file_enc_sha256.to_vec());
            proto.file_length = Some(upload.file_length);
            proto.media_key_timestamp = Some(upload.media_key_timestamp);
            proto.mimetype = Some(
                opts.mimetype
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
            );
            proto.file_name = opts.file_name;
            proto.title = opts.title;
            proto.caption = opts.caption;
            proto.page_count = opts.page_count;
            proto.jpeg_thumbnail = opts.jpeg_thumbnail;
            proto.context_info = opts
                .context_info
                .map(|ci| buffa::MessageField::some(*ci))
                .unwrap_or_default();
            proto
        });
        proto
    }
}

/// Build an audio / voice-note message from an upload result. Carries the
/// streaming sidecar from the upload when present.
pub fn audio_message(upload: UploadResponse, opts: AudioOptions) -> wa::Message {
    {
        let mut proto = wa::Message::default();
        proto.audio_message = buffa::MessageField::some({
            let mut proto = wa::message::AudioMessage::default();
            proto.url = Some(upload.url);
            proto.direct_path = Some(upload.direct_path);
            proto.media_key = Some(upload.media_key.to_vec());
            proto.file_sha256 = Some(upload.file_sha256.to_vec());
            proto.file_enc_sha256 = Some(upload.file_enc_sha256.to_vec());
            proto.file_length = Some(upload.file_length);
            proto.media_key_timestamp = Some(upload.media_key_timestamp);
            proto.streaming_sidecar = upload.streaming_sidecar;
            proto.mimetype = Some(
                opts.mimetype
                    .unwrap_or_else(|| "audio/ogg; codecs=opus".to_string()),
            );
            proto.seconds = opts.duration_seconds;
            proto.ptt = opts.ptt;
            proto.waveform = opts.waveform;
            proto.context_info = opts
                .context_info
                .map(|ci| buffa::MessageField::some(*ci))
                .unwrap_or_default();
            proto
        });
        proto
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
            VideoOptions::default()
                .with_caption("c")
                .with_duration_seconds(12)
                .with_gif_playback(true),
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
            DocumentOptions::default()
                .with_file_name("f.pdf")
                .with_page_count(3),
        )
        .document_message
        .unwrap();
        assert_eq!(doc.file_name.as_deref(), Some("f.pdf"));
        assert_eq!(doc.page_count, Some(3));
        assert_eq!(doc.mimetype.as_deref(), Some("application/octet-stream"));

        let audio = audio_message(
            sample_upload(),
            AudioOptions::default()
                .with_ptt(true)
                .with_duration_seconds(5),
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
            ImageOptions::default().with_context_info(context),
        );

        let image = image_msg.image_message.unwrap();

        assert!(image.context_info.is_set());
    }

    #[test]
    fn video_maps_context_info() {
        let context = Box::new(wa::ContextInfo::default());

        let video_msg = video_message(
            sample_upload(),
            VideoOptions::default().with_context_info(context),
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
