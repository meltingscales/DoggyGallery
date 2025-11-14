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

    /**
     * Initialize pagination info display
     * Looks for elements with id="pagination-info" and data attributes
     */
    function initPaginationInfo() {
        const paginationInfo = document.getElementById('pagination-info');
        if (paginationInfo) {
            const page = parseInt(paginationInfo.dataset.page);
            const perPage = parseInt(paginationInfo.dataset.perPage);
            const entries = parseInt(paginationInfo.dataset.entries);
            const total = parseInt(paginationInfo.dataset.total);

            const start = (page - 1) * perPage + 1;
            const end = Math.min((page - 1) * perPage + entries, total);

            paginationInfo.textContent = `Showing ${start}-${end} of ${total} items`;
        }
    }

    /**
     * Render pagination controls
     * @param {number} currentPage - Current page number
     * @param {number} totalPages - Total number of pages
     * @param {number} perPage - Items per page
     * @returns {HTMLElement} Pagination container element
     */
    function renderPagination(currentPage, totalPages, perPage) {
        const pagination = document.createElement('div');
        pagination.className = 'pagination';

        // Previous button
        if (currentPage > 1) {
            const prevBtn = document.createElement('a');
            prevBtn.href = `?page=${currentPage - 1}&per_page=${perPage}`;
            prevBtn.className = 'pagination-btn';
            prevBtn.textContent = '← Previous';
            pagination.appendChild(prevBtn);
        } else {
            const prevBtn = document.createElement('span');
            prevBtn.className = 'pagination-btn disabled';
            prevBtn.textContent = '← Previous';
            pagination.appendChild(prevBtn);
        }

        // Page numbers container
        const pagesContainer = document.createElement('div');
        pagesContainer.className = 'pagination-pages';

        for (let pageNum = 1; pageNum <= totalPages; pageNum++) {
            if (pageNum === currentPage) {
                const activeBtn = document.createElement('span');
                activeBtn.className = 'pagination-btn active';
                activeBtn.textContent = pageNum;
                pagesContainer.appendChild(activeBtn);
            } else if (
                pageNum === 1 ||
                pageNum === totalPages ||
                (currentPage >= 3 && pageNum >= currentPage - 2 && pageNum <= currentPage + 2) ||
                (currentPage < 3 && pageNum <= 5)
            ) {
                const pageBtn = document.createElement('a');
                pageBtn.href = `?page=${pageNum}&per_page=${perPage}`;
                pageBtn.className = 'pagination-btn';
                pageBtn.textContent = pageNum;
                pagesContainer.appendChild(pageBtn);
            } else if (
                (currentPage >= 4 && pageNum === currentPage - 3) ||
                pageNum === currentPage + 3
            ) {
                const ellipsis = document.createElement('span');
                ellipsis.className = 'pagination-ellipsis';
                ellipsis.textContent = '...';
                pagesContainer.appendChild(ellipsis);
            }
        }

        pagination.appendChild(pagesContainer);

        // Next button
        if (currentPage < totalPages) {
            const nextBtn = document.createElement('a');
            nextBtn.href = `?page=${currentPage + 1}&per_page=${perPage}`;
            nextBtn.className = 'pagination-btn';
            nextBtn.textContent = 'Next →';
            pagination.appendChild(nextBtn);
        } else {
            const nextBtn = document.createElement('span');
            nextBtn.className = 'pagination-btn disabled';
            nextBtn.textContent = 'Next →';
            pagination.appendChild(nextBtn);
        }

        return pagination;
    }

    /**
     * Initialize pagination controls
     * Looks for elements with id="pagination-container" and data attributes
     */
    function initPaginationControls() {
        const container = document.getElementById('pagination-container');
        if (container) {
            const currentPage = parseInt(container.dataset.page);
            const totalPages = parseInt(container.dataset.totalPages);
            const perPage = parseInt(container.dataset.perPage);

            if (totalPages > 1) {
                const pagination = renderPagination(currentPage, totalPages, perPage);
                container.appendChild(pagination);
            }
        }
    }

    // Export utility functions (only if not already defined)
    if (!window.DoggyUtils) {
        window.DoggyUtils = {
            formatBytes,
            getFileIcon,
            escapeHtml,
            createElement,
            initPaginationInfo,
            renderPagination,
            initPaginationControls
        };
    }

    // Auto-initialize pagination on DOMContentLoaded
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            initPaginationInfo();
            initPaginationControls();
        });
    } else {
        initPaginationInfo();
        initPaginationControls();
    }
})();
