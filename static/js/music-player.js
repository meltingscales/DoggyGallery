// Music Player module for DoggyGallery
// Adds playlist features: auto-advance, shuffle mode, play all functionality
(function() {
    'use strict';

    let playlist = [];
    let currentIndex = 0;
    let shuffleMode = false;
    let playAllMode = false;
    let originalPlaylist = [];

    /**
     * Initialize the music player with audio items
     * @param {Array} items - Array of {src: string, type: string} objects
     */
    function initMusicPlayer(items) {
        playlist = items.filter(item => item.type === 'audio');
        originalPlaylist = [...playlist];
        currentIndex = 0;

        // Add event listener for audio ended event to auto-advance
        document.addEventListener('DOMContentLoaded', attachAudioEndedListener);
    }

    /**
     * Attach ended event listener to audio element
     */
    function attachAudioEndedListener() {
        const observer = new MutationObserver(function(mutations) {
            mutations.forEach(function(mutation) {
                mutation.addedNodes.forEach(function(node) {
                    if (node.tagName === 'AUDIO' || node.classList.contains('audio-element')) {
                        node.addEventListener('ended', handleAudioEnded);
                    }
                    // Also check for audio elements within added nodes
                    if (node.querySelector) {
                        const audioElements = node.querySelectorAll('audio, .audio-element');
                        audioElements.forEach(function(audio) {
                            audio.addEventListener('ended', handleAudioEnded);
                        });
                    }
                });
            });
        });

        const lightboxContent = document.getElementById('lightbox-content');
        if (lightboxContent) {
            observer.observe(lightboxContent, { childList: true, subtree: true });
        }

        // Also attach to any existing audio element
        const audio = document.querySelector('#lightbox-content audio, #lightbox-content .audio-element');
        if (audio) {
            audio.addEventListener('ended', handleAudioEnded);
        }
    }

    /**
     * Handle audio ended event - auto-advance to next track
     */
    function handleAudioEnded() {
        if (!playAllMode) return;

        if (shuffleMode) {
            // In shuffle mode, pick random next track
            if (playlist.length <= 1) return;

            let randomIndex;
            do {
                randomIndex = Math.floor(Math.random() * playlist.length);
            } while (randomIndex === currentIndex);

            currentIndex = randomIndex;
        } else {
            // In sequential mode, advance to next track
            currentIndex++;
            if (currentIndex >= playlist.length) {
                currentIndex = 0; // Loop back to beginning
            }
        }

        // Play next track
        const nextTrack = playlist[currentIndex];
        if (nextTrack && window.DoggyLightbox) {
            window.DoggyLightbox.open(nextTrack.src, nextTrack.type);
        }
    }

    /**
     * Play all tracks in random order
     */
    function playAllRandom() {
        if (playlist.length === 0) return;

        playAllMode = true;
        shuffleMode = true;
        updateShuffleButton();

        // Start with a random track
        currentIndex = Math.floor(Math.random() * playlist.length);
        const track = playlist[currentIndex];

        if (track && window.DoggyLightbox) {
            window.DoggyLightbox.open(track.src, track.type);
        }
    }

    /**
     * Play all tracks sequentially
     */
    function playAllSequential() {
        if (playlist.length === 0) return;

        playAllMode = true;
        shuffleMode = false;
        updateShuffleButton();

        // Start with first track
        currentIndex = 0;
        const track = playlist[currentIndex];

        if (track && window.DoggyLightbox) {
            window.DoggyLightbox.open(track.src, track.type);
        }
    }

    /**
     * Toggle shuffle mode
     */
    function toggleShuffle() {
        shuffleMode = !shuffleMode;
        updateShuffleButton();
    }

    /**
     * Update the shuffle button text
     */
    function updateShuffleButton() {
        const button = document.getElementById('shuffle-btn');
        if (button) {
            button.textContent = shuffleMode ? '🔀 Shuffle: ON' : '🔀 Shuffle: OFF';
        }
    }

    /**
     * Stop play all mode
     */
    function stopPlayAll() {
        playAllMode = false;
    }

    // Export functions
    if (!window.MusicPlayer) {
        window.MusicPlayer = {
            init: initMusicPlayer,
            playAllRandom: playAllRandom,
            playAllSequential: playAllSequential,
            toggleShuffle: toggleShuffle,
            stopPlayAll: stopPlayAll
        };
    }

    // Override DoggyLightbox.close to also stop play all mode
    if (window.DoggyLightbox) {
        const originalClose = window.DoggyLightbox.close;
        window.DoggyLightbox.close = function() {
            stopPlayAll();
            originalClose();
        };
    }
})();
