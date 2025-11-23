use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures::StreamExt;
use uuid::Uuid;
use actix_multipart::Multipart;

pub async fn save_video(mut payload: Multipart) -> Result<String, actix_web::Error> {
    let mut file_path = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_type = field.content_disposition();
        let filename = content_type.get_filename().unwrap_or("video.mp4");

        let sanitized_filename = sanitize_filename::sanitize(filename);
        let unique_name = format!("{}-{}", Uuid::new_v4(), sanitized_filename);
        let path = format!("uploads/{}", unique_name);
        file_path = path.clone();

        let mut f = fs::File::create(&path).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f.write_all(&data).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        }
    }

    // Trigger mocked multi-resolution processing
    // In production, you would run this in a separate thread or job queue
    let process_path = file_path.clone();
    tokio::spawn(async move {
        mock_process_video_multi_res(&process_path).await;
    });

    Ok(file_path)
}

// Mock function to simulate video processing logic for multiple resolutions
async fn mock_process_video_multi_res(input_path: &str) {
    println!("Starting processing for: {}", input_path);

    // Simulate processing time
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // In a real scenario with ffmpeg, you would generate:
    // - 1080p version
    // - 720p version
    // - 480p version
    // - HLS playlist (master.m3u8) linking them

    println!("Generated 1080p for {}", input_path);
    println!("Generated 720p for {}", input_path);
    println!("Generated 480p for {}", input_path);
    println!("Generated HLS Playlist for {}", input_path);

    // Example ffmpeg command for HLS:
    // ffmpeg -i input.mp4 \
    //   -map 0:v:0 -map 0:a:0 -map 0:v:0 -map 0:a:0 \
    //   -c:v:0 libx264 -b:v:0 3000k -s:v:0 1920x1080 -profile:v:0 high \
    //   -c:v:1 libx264 -b:v:1 1500k -s:v:1 1280x720 -profile:v:1 main \
    //   -f hls -var_stream_map "v:0,a:0 v:1,a:1" master.m3u8
}
