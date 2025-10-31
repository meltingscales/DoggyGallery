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
            const audio = document.createElement('audio');
            audio.controls = true;
            audio.autoplay = true;
            audio.style.width = '100%';
            audio.style.maxWidth = '600px';
            const source = document.createElement('source');
            source.src = item.src;
            audio.appendChild(source);
            content.appendChild(audio);
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
