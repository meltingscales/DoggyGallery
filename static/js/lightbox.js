// Lightbox viewer module for DoggyGallery
(function() {
    'use strict';

    let mediaItems = [];
    let currentIndex = 0;
    let touchStartX = 0;
    let touchEndX = 0;

    /**
     * Initialize the lightbox with media items
     * @param {Array} items - Array of {src: string, type: string} objects
     */
    function initLightbox(items) {
        mediaItems = items || [];
        currentIndex = 0;

        // Add keyboard event listener
        document.addEventListener('keydown', handleKeyboard);

        // Add touch event listeners for swipe
        const lightboxContent = document.getElementById('lightbox-content');
        if (lightboxContent) {
            lightboxContent.addEventListener('touchstart', handleTouchStart, false);
            lightboxContent.addEventListener('touchend', handleTouchEnd, false);
            lightboxContent.onclick = function(e) {
                e.stopPropagation();
            };
        }
    }

    /**
     * Open lightbox with specific media
     * @param {string} src - Media source URL
     * @param {string} type - Media type (image, video, audio)
     */
    function openLightbox(src, type) {
        // Find the index of this media item
        currentIndex = mediaItems.findIndex(item => item.src === src);
        if (currentIndex === -1) currentIndex = 0;

        displayMedia(currentIndex);
        const lightbox = document.getElementById('lightbox');
        if (lightbox) {
            lightbox.classList.add('active');
        }
    }

    /**
     * Display media at given index
     * @param {number} index - Index of media to display
     */
    function displayMedia(index) {
        if (mediaItems.length === 0) return;

        // Wrap around if out of bounds
        if (index < 0) index = mediaItems.length - 1;
        if (index >= mediaItems.length) index = 0;

        currentIndex = index;
        const item = mediaItems[currentIndex];
        const content = document.getElementById('lightbox-content');
        if (!content) return;

        content.innerHTML = '';

        if (item.type === 'image') {
            const img = document.createElement('img');
            img.src = item.src;
            content.appendChild(img);
        } else if (item.type === 'video') {
            const video = document.createElement('video');
            video.controls = true;
            video.autoplay = true;
            const source = document.createElement('source');
            source.src = item.src;
            video.appendChild(source);
            content.appendChild(video);
        } else if (item.type === 'audio') {
            createEnhancedAudioPlayer(item.src, content);
        }
    }

    /**
     * Close the lightbox
     */
    function closeLightbox() {
        const lightbox = document.getElementById('lightbox');
        const content = document.getElementById('lightbox-content');
        if (lightbox) {
            lightbox.classList.remove('active');
        }
        if (content) {
            content.innerHTML = '';
        }
    }

    /**
     * Navigate to next media
     */
    function nextMedia() {
        displayMedia(currentIndex + 1);
    }

    /**
     * Navigate to previous media
     */
    function prevMedia() {
        displayMedia(currentIndex - 1);
    }

    /**
     * Navigate to random media
     */
    function randomMedia() {
        if (mediaItems.length <= 1) return;

        let randomIndex;
        do {
            randomIndex = Math.floor(Math.random() * mediaItems.length);
        } while (randomIndex === currentIndex);

        displayMedia(randomIndex);
    }

    /**
     * Handle keyboard navigation
     * @param {KeyboardEvent} e - Keyboard event
     */
    function handleKeyboard(e) {
        const lightbox = document.getElementById('lightbox');
        if (!lightbox || !lightbox.classList.contains('active')) return;

        if (e.key === 'ArrowRight') {
            e.preventDefault();
            nextMedia();
        } else if (e.key === 'ArrowLeft') {
            e.preventDefault();
            prevMedia();
        } else if (e.key === 'Escape') {
            e.preventDefault();
            closeLightbox();
        }
    }

    /**
     * Handle touch start event
     * @param {TouchEvent} e - Touch event
     */
    function handleTouchStart(e) {
        touchStartX = e.changedTouches[0].screenX;
    }

    /**
     * Handle touch end event
     * @param {TouchEvent} e - Touch event
     */
    function handleTouchEnd(e) {
        touchEndX = e.changedTouches[0].screenX;
        handleSwipe();
    }

    /**
     * Handle swipe gesture
     */
    function handleSwipe() {
        const swipeThreshold = 50; // Minimum distance for a swipe
        const diff = touchStartX - touchEndX;

        if (Math.abs(diff) > swipeThreshold) {
            if (diff > 0) {
                // Swiped left, show next
                nextMedia();
            } else {
                // Swiped right, show previous
                prevMedia();
            }
        }
    }

    /**
     * Create an enhanced audio player with album art, volume, and scrubbing
     * @param {string} src - Audio source URL
     * @param {HTMLElement} container - Container to append player to
     */
    function createEnhancedAudioPlayer(src, container) {
        // Create player container
        const playerContainer = document.createElement('div');
        playerContainer.className = 'enhanced-audio-player';

        // Get track name from URL
        const trackName = decodeURIComponent(src.split('/').pop().split('?')[0]);

        // Get directory path for album art search
        const pathParts = src.split('/');
        const dirPath = pathParts.slice(0, -1).join('/');

        // Create album art container
        const albumArtContainer = document.createElement('div');
        albumArtContainer.className = 'album-art-container';

        const albumArt = document.createElement('img');
        albumArt.className = 'album-art';
        albumArt.src = '/static/img/default-album-art.png';
        albumArt.alt = 'Album Art';

        // Try to find album art in the same directory
        tryLoadAlbumArt(dirPath, albumArt);

        albumArtContainer.appendChild(albumArt);

        // Create track info
        const trackInfo = document.createElement('div');
        trackInfo.className = 'track-info';
        trackInfo.textContent = trackName;

        // Create audio element (hidden)
        const audio = document.createElement('audio');
        audio.className = 'audio-element';
        audio.src = src;
        audio.autoplay = true;
        audio.preload = 'metadata'; // Enable seeking by preloading metadata

        // Create controls container
        const controlsContainer = document.createElement('div');
        controlsContainer.className = 'audio-controls';

        // Time display
        const timeDisplay = document.createElement('div');
        timeDisplay.className = 'time-display';
        timeDisplay.textContent = '0:00 / 0:00';

        // Seek slider
        const seekSlider = document.createElement('input');
        seekSlider.type = 'range';
        seekSlider.min = '0';
        seekSlider.max = '100';
        seekSlider.value = '0';
        seekSlider.className = 'seek-slider';

        // Play/Pause button
        const playPauseBtn = document.createElement('button');
        playPauseBtn.className = 'play-pause-btn';
        playPauseBtn.innerHTML = '⏸';
        playPauseBtn.onclick = (e) => {
            e.stopPropagation();
            if (audio.paused) {
                audio.play();
                playPauseBtn.innerHTML = '⏸';
            } else {
                audio.pause();
                playPauseBtn.innerHTML = '▶';
            }
        };

        // Volume container
        const volumeContainer = document.createElement('div');
        volumeContainer.className = 'volume-container';

        const volumeIcon = document.createElement('span');
        volumeIcon.className = 'volume-icon';
        volumeIcon.textContent = '🔊';

        const volumeSlider = document.createElement('input');
        volumeSlider.type = 'range';
        volumeSlider.min = '0';
        volumeSlider.max = '100';
        volumeSlider.value = '100';
        volumeSlider.className = 'volume-slider';

        // Event listeners
        audio.addEventListener('loadedmetadata', () => {
            seekSlider.max = audio.duration;
            updateTimeDisplay();
        });

        audio.addEventListener('timeupdate', () => {
            if (!seekSlider.dataset.seeking) {
                seekSlider.value = audio.currentTime;
            }
            updateTimeDisplay();
        });

        seekSlider.addEventListener('mousedown', () => {
            seekSlider.dataset.seeking = 'true';
        });

        seekSlider.addEventListener('mouseup', () => {
            delete seekSlider.dataset.seeking;
        });

        seekSlider.addEventListener('input', () => {
            audio.currentTime = seekSlider.value;
        });

        volumeSlider.addEventListener('input', () => {
            audio.volume = volumeSlider.value / 100;
            updateVolumeIcon();
        });

        function updateTimeDisplay() {
            const current = formatTime(audio.currentTime);
            const total = formatTime(audio.duration);
            timeDisplay.textContent = `${current} / ${total}`;
        }

        function formatTime(seconds) {
            if (isNaN(seconds)) return '0:00';
            const mins = Math.floor(seconds / 60);
            const secs = Math.floor(seconds % 60);
            return `${mins}:${secs.toString().padStart(2, '0')}`;
        }

        function updateVolumeIcon() {
            const volume = audio.volume;
            if (volume === 0) {
                volumeIcon.textContent = '🔇';
            } else if (volume < 0.5) {
                volumeIcon.textContent = '🔉';
            } else {
                volumeIcon.textContent = '🔊';
            }
        }

        // Assemble controls
        volumeContainer.appendChild(volumeIcon);
        volumeContainer.appendChild(volumeSlider);

        const playbackControls = document.createElement('div');
        playbackControls.className = 'playback-controls';
        playbackControls.appendChild(playPauseBtn);
        playbackControls.appendChild(volumeContainer);

        controlsContainer.appendChild(timeDisplay);
        controlsContainer.appendChild(seekSlider);
        controlsContainer.appendChild(playbackControls);

        // Assemble player
        playerContainer.appendChild(albumArtContainer);
        playerContainer.appendChild(trackInfo);
        playerContainer.appendChild(audio);
        playerContainer.appendChild(controlsContainer);

        container.appendChild(playerContainer);

        // Store reference for ended event handling
        container.audioElement = audio;
    }

    /**
     * Try to load album art from the same directory
     * @param {string} dirPath - Directory path
     * @param {HTMLImageElement} imgElement - Image element to update
     */
    function tryLoadAlbumArt(dirPath, imgElement) {
        const commonNames = ['cover.jpg', 'cover.png', 'folder.jpg', 'folder.png', 'album.jpg', 'album.png'];

        let found = false;

        // Check if this is an archive path
        const isArchivePath = dirPath.includes('!/');

        async function tryNext(index) {
            if (index >= commonNames.length || found) return;

            let artPath;
            if (isArchivePath) {
                // For archive paths, replace the filename after !/ with cover image name
                const parts = dirPath.split('!/');
                const archivePath = parts[0];
                const internalDir = parts[1].substring(0, parts[1].lastIndexOf('/'));
                artPath = `${archivePath}!/${internalDir}/${commonNames[index]}`;
            } else {
                artPath = `${dirPath}/${commonNames[index]}`;
            }

            // Try to load the image
            const testImg = new Image();
            testImg.onload = () => {
                if (!found) {
                    found = true;
                    imgElement.src = artPath;
                }
            };
            testImg.onerror = () => {
                tryNext(index + 1);
            };
            testImg.src = artPath;
        }

        tryNext(0);
    }

    // Export functions (only if not already defined)
    if (!window.DoggyLightbox) {
        window.DoggyLightbox = {
            init: initLightbox,
            open: openLightbox,
            close: closeLightbox,
            next: nextMedia,
            prev: prevMedia,
            random: randomMedia
        };
    }
})();
