// API client module for DoggyGallery
(function() {
    'use strict';

    /**
     * Fetch configuration from the server
     * @returns {Promise<Object>} Configuration object
     */
    async function fetchConfig() {
        const response = await fetch('/api/config');
        if (!response.ok) {
            throw new Error(`Failed to fetch config: ${response.statusText}`);
        }
        return await response.json();
    }

    /**
     * Search/filter media files
     * @param {Object} filters - Filter parameters
     * @param {string} [filters.type] - File type (image, video, audio)
     * @param {string} [filters.extension] - File extension
     * @param {string} [filters.name] - File name (fuzzy match)
     * @returns {Promise<Object>} Search results
     */
    async function searchMedia(filters = {}) {
        const params = new URLSearchParams();

        if (filters.type) params.append('type', filters.type);
        if (filters.extension) params.append('extension', filters.extension);
        if (filters.name) params.append('name', filters.name);

        const response = await fetch(`/api/filter?${params.toString()}`);
        if (!response.ok) {
            throw new Error(`Search failed: ${response.statusText}`);
        }
        return await response.json();
    }

    // Export functions for use in other modules (only if not already defined)
    if (!window.DoggyAPI) {
        window.DoggyAPI = {
            fetchConfig,
            searchMedia
        };
    }
})();
