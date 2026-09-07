# Pomodoro Break Video

## Objective

Allow the user to configure a video URL that is displayed during
the Pomodoro break.

## Requirements

### Video source

When a video URL is configured for the break:

- The application must extract/use the video identifier from the URL.
- The application must display the video using YouTube's embed player.
- The application must NOT load the complete YouTube website.
- The WebView must contain only the embedded video/player.
- Navigation to the normal YouTube website must not be required.

### Break behavior

When a break starts and a video URL is configured:

1. The embedded video must be displayed.
2. The video must occupy the available screen in fullscreen mode.
3. The Pomodoro countdown must not be visible while the video is being displayed.
4. The user must not see the application's normal break UI over the video.
5. The video must remain within the application's WebView/window.

### URL handling

The implementation must support the URL formats currently accepted
by the application.

The implementation must not assume that the URL is already an embed URL.

Examples of URLs that may need to be handled:

- https://www.youtube.com/watch?v=VIDEO_ID
- https://youtu.be/VIDEO_ID
- https://www.youtube.com/shorts/VIDEO_ID

### Important restriction

Do not implement this by opening youtube.com inside the WebView.

Use the YouTube embedded player mechanism instead.