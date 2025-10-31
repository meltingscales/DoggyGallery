// Utility functions for DoggyGallery
(function() {
    'use strict';

    /**
     * Format bytes to human-readable size
     * @param {number} bytes - Size in bytes
     * @returns {string} Formatted size string
     */
    function formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    /**
     * Get icon emoji for file type
     * @param {string} type - File type (image, video, audio)
     * @returns {string} Icon emoji
     */
    function getFileIcon(type) {
        const icons = {
            'image': '🖼️',
            'video': '🎬',
            'audio': '🎵'
        };
        return icons[type] || '📄';
    }

    /**
     * Escape HTML to prevent XSS
     * @param {string} text - Text to escape
     * @returns {string} Escaped text
     */
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Create an element with attributes and children
     * @param {string} tag - HTML tag name
     * @param {Object} attrs - Attributes object
     * @param {Array|string} children - Child elements or text
     * @returns {HTMLElement} Created element
     */
    function createElement(tag, attrs = {}, children = []) {
        const el = document.createElement(tag);

        Object.entries(attrs).forEach(([key, value]) => {
            if (key === 'className') {
                el.className = value;
            } else if (key === 'style' && typeof value === 'object') {
                Object.assign(el.style, value);
            } else if (key.startsWith('on')) {
                el.addEventListener(key.slice(2).toLowerCase(), value);
            } else {
                el.setAttribute(key, value);
            }
        });

        if (typeof children === 'string') {
            el.textContent = children;
        } else if (Array.isArray(children)) {
            children.forEach(child => {
                if (typeof child === 'string') {
                    el.appendChild(document.createTextNode(child));
                } else if (child instanceof HTMLElement) {
                    el.appendChild(child);
                }
            });
        }

        return el;
    }

    // Export utility functions (only if not already defined)
    if (!window.DoggyUtils) {
        window.DoggyUtils = {
            formatBytes,
            getFileIcon,
            escapeHtml,
            createElement
        };
    }
})();
